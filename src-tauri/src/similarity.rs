#[cfg(not(has_proprietary_ai))]
use std::collections::HashMap;
#[cfg(not(has_proprietary_ai))]
use rayon::prelude::*;

use crate::decoder::ImageDecoder;
use crate::models::{PhotoGroup, SimilarityConfig, SubjectCluster};
use crate::npu_engine::InferenceTelemetry;
#[cfg(not(has_proprietary_ai))]
use crate::npu_engine::{ComputeBackend, NpuEngine};

/// 图像感知哈希与同主体/去重分析器 (Open-Core 接口层)
pub struct SimilarityDetector;

impl SimilarityDetector {
    /// 计算 64-bit 差分感知哈希 (dHash)
    pub fn compute_dhash_from_image(img: &image::DynamicImage) -> u64 {
        let gray = img
            .resize_exact(9, 8, image::imageops::FilterType::Nearest)
            .to_luma8();
        let mut hash = 0u64;
        for row in 0..8 {
            for col in 0..8 {
                let left = gray.get_pixel(col, row)[0];
                let right = gray.get_pixel(col + 1, row)[0];
                if left > right {
                    hash |= 1u64 << (row * 8 + col);
                }
            }
        }
        hash
    }

    /// 从原始图片字节流解析并计算 dHash
    pub fn compute_dhash_from_bytes(bytes: &[u8]) -> Option<u64> {
        let img = image::load_from_memory(bytes).ok()?;
        Some(Self::compute_dhash_from_image(&img))
    }

    /// 为单张照片计算/提取 dHash
    pub fn compute_dhash_for_photo(photo: &PhotoGroup) -> Option<u64> {
        if let Some(ref hex) = photo.phash {
            if let Ok(val) = u64::from_str_radix(hex, 16) {
                return Some(val);
            }
        }

        let target_path = photo.std_image.as_ref().or(photo.raw_image.as_ref())?;
        if let Ok(thumb_bytes) = ImageDecoder::extract_thumbnail_bytes(target_path) {
            return Self::compute_dhash_from_bytes(&thumb_bytes);
        }

        None
    }

    /// 计算两个 64-bit dHash 的海明距离 (Hamming Distance: 0-64)
    #[inline(always)]
    pub fn hamming_distance(h1: u64, h2: u64) -> u32 {
        (h1 ^ h2).count_ones()
    }

    /// 执行严格去重与同主体筛选聚类
    pub fn detect_duplicates_and_subjects(
        photos: &mut [PhotoGroup],
        config: &SimilarityConfig,
    ) -> Vec<SubjectCluster> {
        let (clusters, _) = Self::detect_with_telemetry(photos, config);
        clusters
    }

    /// 执行视觉聚类与去重分析 (当存在闭源 NPU/AI 内核时自动调用神经网络聚类，否则回退至基础 dHash)
    pub fn detect_with_telemetry(
        photos: &mut [PhotoGroup],
        config: &SimilarityConfig,
    ) -> (Vec<SubjectCluster>, InferenceTelemetry) {
        #[cfg(has_proprietary_ai)]
        {
            crate::npu_engine::kernel::detect_similarity_proprietary(photos, config)
        }
        #[cfg(not(has_proprietary_ai))]
        {
            Self::detect_open_core_fallback(photos, config)
        }
    }

    #[cfg(not(has_proprietary_ai))]
    fn detect_open_core_fallback(
        photos: &mut [PhotoGroup],
        config: &SimilarityConfig,
    ) -> (Vec<SubjectCluster>, InferenceTelemetry) {
        for p in photos.iter_mut() {
            p.is_duplicate = false;
            p.duplicate_of = None;
            p.subject_group_id = None;
            p.is_subject_best = false;
        }

        let requested_backend = ComputeBackend::from_str_opt(config.compute_backend.as_deref());
        let (_, telemetry) = NpuEngine::extract_batch_features(photos, requested_backend);

        if photos.len() < config.min_group_count {
            return (Vec::new(), telemetry);
        }

        let computed_hashes: Vec<Option<u64>> = photos
            .par_iter()
            .map(|p| Self::compute_dhash_for_photo(p))
            .collect();

        for (idx, hash_opt) in computed_hashes.iter().enumerate() {
            if let Some(h) = hash_opt {
                photos[idx].phash = Some(format!("{:016x}", h));
            }
        }

        let mut dir_to_indices: HashMap<String, Vec<usize>> = HashMap::new();
        for (idx, p) in photos.iter().enumerate() {
            let dir_key = p.folder_path.to_string_lossy().to_string();
            dir_to_indices.entry(dir_key).or_default().push(idx);
        }

        let mut clusters = Vec::new();
        let mut global_cluster_idx = 1usize;

        for (_dir, indices) in dir_to_indices {
            if indices.len() < config.min_group_count {
                continue;
            }

            let mut items: Vec<(usize, i64, Option<u64>)> = indices
                .into_iter()
                .map(|idx| {
                    let p = &photos[idx];
                    let sub_sec = p.exif.as_ref().and_then(|e| e.sub_sec_time.as_deref());
                    let time_ms = crate::burst::BurstDetector::parse_timestamp_ms(
                        p.capture_time.as_deref(),
                        sub_sec,
                    )
                    .unwrap_or(0);
                    let h = computed_hashes[idx];
                    (idx, time_ms, h)
                })
                .collect();

            items.sort_by(|a, b| a.1.cmp(&b.1));

            let mut is_dup_idx = vec![false; items.len()];
            for i in 0..items.len() {
                if is_dup_idx[i] {
                    continue;
                }
                let (idx_i, time_i, Some(h_i)) = items[i] else {
                    continue;
                };
                for j in (i + 1)..items.len() {
                    if is_dup_idx[j] {
                        continue;
                    }
                    let (idx_j, time_j, Some(h_j)) = items[j] else {
                        continue;
                    };
                    let time_diff_secs = (time_j - time_i).abs() / 1000;
                    if time_diff_secs <= 30
                        && Self::hamming_distance(h_i, h_j) <= config.dedup_threshold
                    {
                        is_dup_idx[j] = true;
                        photos[idx_j].is_duplicate = true;
                        photos[idx_j].duplicate_of = Some(photos[idx_i].id.clone());
                        if config.auto_reject_duplicates && photos[idx_j].flag == 0 {
                            photos[idx_j].flag = -1;
                        }
                    }
                }
            }

            let mut visited = vec![false; items.len()];
            for i in 0..items.len() {
                if visited[i] {
                    continue;
                }
                let mut current_group: Vec<usize> = vec![items[i].0];
                visited[i] = true;

                let (_base_idx, base_time, Some(h_lead)) = items[i] else {
                    continue;
                };
                let mut last_time = base_time;

                for j in (i + 1)..items.len() {
                    if visited[j] {
                        continue;
                    }
                    let (candidate_idx, cand_time, Some(h_cand)) = items[j] else {
                        continue;
                    };
                    let diff_from_last = (cand_time - last_time).abs() / 1000;
                    let diff_from_start = (cand_time - base_time).abs() / 1000;

                    if (diff_from_last as u64) <= config.time_window_secs
                        && (diff_from_start as u64) <= config.time_window_secs * 3
                        && Self::hamming_distance(h_lead, h_cand) <= config.subject_threshold
                    {
                        current_group.push(candidate_idx);
                        visited[j] = true;
                        last_time = cand_time;
                    }
                }

                if current_group.len() >= config.min_group_count {
                    let group_id = format!("subj_{:03}", global_cluster_idx);
                    let photo_ids: Vec<String> =
                        current_group.iter().map(|&idx| photos[idx].id.clone()).collect();
                    let count = photo_ids.len();
                    let start_time = photos[current_group[0]].capture_time.clone();
                    let end_time = photos[*current_group.last().unwrap()].capture_time.clone();
                    let best_id = photos[current_group[0]].id.clone();

                    let mut duplicate_ids = Vec::new();
                    for &idx in &current_group {
                        let p = &mut photos[idx];
                        p.subject_group_id = Some(group_id.clone());
                        p.is_subject_best = p.id == best_id;
                        if p.is_subject_best && config.auto_flag_best && p.flag == 0 {
                            p.flag = 1;
                        }
                        if p.is_duplicate {
                            duplicate_ids.push(p.id.clone());
                        }
                    }

                    clusters.push(SubjectCluster {
                        group_id,
                        cluster_type: "subject".to_string(),
                        photo_ids,
                        count,
                        best_photo_id: Some(best_id),
                        duplicate_ids,
                        start_time,
                        end_time,
                        time_span_secs: 0,
                    });
                    global_cluster_idx += 1;
                }
            }
        }

        (clusters, telemetry)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn make_test_photo(id: &str, name: &str, phash: &str, time: &str, size: u64) -> PhotoGroup {
        let mut p = PhotoGroup::new(id.to_string(), name.to_string(), PathBuf::from("C:/photos"));
        p.phash = Some(phash.to_string());
        p.capture_time = Some(time.to_string());
        p.file_size_bytes = size;
        p
    }

    #[test]
    fn test_hamming_distance() {
        let h1 = 0b11110000u64;
        let h2 = 0b11110011u64;
        assert_eq!(SimilarityDetector::hamming_distance(h1, h2), 2);
    }

    #[test]
    fn test_similarity_clustering_and_dedup() {
        let mut photos = vec![
            make_test_photo("1", "DSC_0001", "ffff0000ffff0000", "2024-05-10 10:00:00", 5000),
            make_test_photo("2", "DSC_0002", "ffff0000ffff0001", "2024-05-10 10:00:02", 4800),
            make_test_photo("3", "DSC_0003", "ffff0000ffff000f", "2024-05-10 10:00:15", 5500),
            make_test_photo("4", "DSC_0004", "0000ffff0000ffff", "2024-05-10 10:00:20", 6000),
        ];

        let config = SimilarityConfig {
            dedup_threshold: 3,
            subject_threshold: 12,
            time_window_secs: 60,
            min_group_count: 2,
            auto_reject_duplicates: true,
            auto_flag_best: true,
            compute_backend: None,
            auto_tag_scene: false,
        };

        let clusters = SimilarityDetector::detect_duplicates_and_subjects(&mut photos, &config);

        assert_eq!(photos[1].is_duplicate, true);
        assert_eq!(photos[1].duplicate_of, Some("1".to_string()));
        assert_eq!(photos[1].flag, -1);
        assert_eq!(clusters.len(), 1);
        assert_eq!(clusters[0].count, 3);
        assert_eq!(photos[3].subject_group_id, None);
    }
}
