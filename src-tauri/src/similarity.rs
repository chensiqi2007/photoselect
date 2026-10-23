use std::collections::HashMap;
use rayon::prelude::*;

use crate::decoder::ImageDecoder;
use crate::models::{PhotoGroup, SimilarityConfig, SubjectCluster};

/// 图像感知哈希与同主体/去重分析器
pub struct SimilarityDetector;

impl SimilarityDetector {
    /// 计算 64-bit 差分感知哈希 (dHash)
    /// 算法：将图像缩放为 9x8 灰度图，比较同一行相邻像素亮度 (8x8 = 64 位)
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

    /// 从原始图片字节流（缩略图或 JPEG）解析并计算 dHash
    pub fn compute_dhash_from_bytes(bytes: &[u8]) -> Option<u64> {
        let img = image::load_from_memory(bytes).ok()?;
        Some(Self::compute_dhash_from_image(&img))
    }

    /// 为单张照片计算/提取 dHash (首选轻量内嵌缩略图，0.5ms 极速解析)
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
        if photos.len() < config.min_group_count {
            return Vec::new();
        }

        // 1. 并发并行计算所有照片的 dHash (若尚未缓存)
        let computed_hashes: Vec<Option<u64>> = photos
            .par_iter()
            .map(|p| Self::compute_dhash_for_photo(p))
            .collect();

        // 写回 phash 字符串
        for (idx, hash_opt) in computed_hashes.iter().enumerate() {
            if let Some(h) = hash_opt {
                photos[idx].phash = Some(format!("{:016x}", h));
            }
            // 重置既有标记
            photos[idx].is_duplicate = false;
            photos[idx].duplicate_of = None;
            photos[idx].subject_group_id = None;
            photos[idx].is_subject_best = false;
        }

        // 2. 按目录隔离分组处理
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

            // 预提取时间戳与哈希以按拍摄时序处理
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

            // 3. 严格去重判定 (Strict Deduplication)
            // 邻近或同场景下海明距离 <= dedup_threshold (通常 <= 3) 视为重复连击/雷同帧
            let mut master_to_duplicates: HashMap<usize, Vec<usize>> = HashMap::new();
            let mut is_dup_idx = vec![false; items.len()];

            for i in 0..items.len() {
                if is_dup_idx[i] {
                    continue;
                }
                let (idx_i, time_i, hash_i) = items[i];
                let h_i = match hash_i {
                    Some(h) => h,
                    None => continue,
                };

                for j in (i + 1)..items.len() {
                    if is_dup_idx[j] {
                        continue;
                    }
                    let (idx_j, time_j, hash_j) = items[j];
                    let h_j = match hash_j {
                        Some(h) => h,
                        None => continue,
                    };

                    // 时间跨度在 30 秒内且视觉海明距离 <= dedup_threshold
                    let time_diff_secs = (time_j - time_i).abs() / 1000;
                    if time_diff_secs <= 30 && Self::hamming_distance(h_i, h_j) <= config.dedup_threshold {
                        is_dup_idx[j] = true;
                        master_to_duplicates.entry(idx_i).or_default().push(idx_j);

                        // 标记次选照片为重复
                        photos[idx_j].is_duplicate = true;
                        photos[idx_j].duplicate_of = Some(photos[idx_i].id.clone());

                        if config.auto_reject_duplicates && photos[idx_j].flag == 0 {
                            photos[idx_j].flag = -1;
                        }
                    }
                }
            }

            // 4. 同主体聚类 (Same-Subject Clustering)
            // 采用自适应时序滑动窗口聚类：时间跨度在 time_window_secs 内，且主体构图相似度高 (海明距离 <= subject_threshold)
            let mut visited = vec![false; items.len()];

            for i in 0..items.len() {
                if visited[i] {
                    continue;
                }

                let mut current_group: Vec<usize> = vec![items[i].0];
                visited[i] = true;

                let (_base_idx, base_time, base_hash) = items[i];
                let lead_hash = base_hash;

                if let Some(h_lead) = lead_hash {
                    let mut last_time = base_time;

                    for j in (i + 1)..items.len() {
                        if visited[j] {
                            continue;
                        }

                        let (candidate_idx, cand_time, cand_hash) = items[j];
                        if let Some(h_cand) = cand_hash {
                            let diff_from_last = (cand_time - last_time).abs() / 1000;
                            let diff_from_start = (cand_time - base_time).abs() / 1000;

                            // 如果与前一张时间相隔在 time_window_secs 内，且海明距离符合同主体阈值
                            if (diff_from_last as u64) <= config.time_window_secs
                                && (diff_from_start as u64) <= config.time_window_secs * 3
                                && Self::hamming_distance(h_lead, h_cand) <= config.subject_threshold
                            {
                                current_group.push(candidate_idx);
                                visited[j] = true;
                                last_time = cand_time;
                            }
                        }
                    }
                }

                // 达到最小组建数量，固化为一个同主体组
                if current_group.len() >= config.min_group_count {
                    let group_id = format!("subj_{:03}", global_cluster_idx);
                    let photo_ids: Vec<String> = current_group
                        .iter()
                        .map(|&idx| photos[idx].id.clone())
                        .collect();
                    let count = photo_ids.len();

                    let start_time = photos[current_group[0]].capture_time.clone();
                    let end_time = photos[*current_group.last().unwrap()].capture_time.clone();

                    // 评选同主体组内 Best Shot
                    let mut best_idx = current_group[count / 2];
                    let mut best_score = -1i64;

                    for (order, &idx) in current_group.iter().enumerate() {
                        let p = &photos[idx];
                        let mut score = 0i64;
                        score += (p.rating as i64) * 1_000_000_000;
                        if p.flag == 1 {
                            score += 500_000_000;
                        } else if p.flag == -1 {
                            score -= 500_000_000;
                        }
                        // 重复帧扣分
                        if p.is_duplicate {
                            score -= 100_000_000;
                        }
                        score += p.file_size_bytes as i64;
                        let dist_from_center = (order as i64 - (count as i64 / 2)).abs();
                        score -= dist_from_center * 200;

                        if score > best_score {
                            best_score = score;
                            best_idx = idx;
                        }
                    }

                    let best_id = photos[best_idx].id.clone();

                    // 写回照片属性
                    let mut duplicate_ids = Vec::new();
                    for &idx in &current_group {
                        let p = &mut photos[idx];
                        p.subject_group_id = Some(group_id.clone());
                        let is_best = p.id == best_id;
                        p.is_subject_best = is_best;

                        if is_best && config.auto_flag_best && p.flag == 0 {
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

        clusters
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
            // 基础帧
            make_test_photo("1", "DSC_0001", "ffff0000ffff0000", "2024-05-10 10:00:00", 5000),
            // 严格重复帧 (仅差 1 位)
            make_test_photo("2", "DSC_0002", "ffff0000ffff0001", "2024-05-10 10:00:02", 4800),
            // 同主体帧 (差 4 位，在 12 位阈值内)
            make_test_photo("3", "DSC_0003", "ffff0000ffff000f", "2024-05-10 10:00:15", 5500),
            // 完全不相干的另一主体 (海明距离 > 30)
            make_test_photo("4", "DSC_0004", "0000ffff0000ffff", "2024-05-10 10:00:20", 6000),
        ];

        let config = SimilarityConfig {
            dedup_threshold: 3,
            subject_threshold: 12,
            time_window_secs: 60,
            min_group_count: 2,
            auto_reject_duplicates: true,
            auto_flag_best: true,
        };

        let clusters = SimilarityDetector::detect_duplicates_and_subjects(&mut photos, &config);

        // 照片 2 应该被判为照片 1 的严格重复，且自动标记为 Reject (-1)
        assert_eq!(photos[1].is_duplicate, true);
        assert_eq!(photos[1].duplicate_of, Some("1".to_string()));
        assert_eq!(photos[1].flag, -1);

        // 照片 1, 2, 3 构成同主体组
        assert_eq!(clusters.len(), 1);
        assert_eq!(clusters[0].count, 3);
        // 照片 4 距离太远，不构成组
        assert_eq!(photos[3].subject_group_id, None);
    }
}
