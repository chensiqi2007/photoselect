use chrono::NaiveDateTime;
use std::cmp::Ordering;
use std::collections::HashMap;

use crate::models::{BurstCluster, BurstConfig, PhotoGroup};

/// 单张照片在连拍分析中的瞬时元数据条目
struct ItemMeta {
    idx: usize,
    time_ms: Option<i64>,
    seq_num: Option<u64>,
    capture_time: Option<String>,
}

/// 毫秒级高速连拍聚合分析器 (移植与自研优化)
pub struct BurstDetector;

impl BurstDetector {
    /// 解析时间戳为相对纪元毫秒数
    pub fn parse_timestamp_ms(time_str: Option<&str>, sub_sec: Option<&str>) -> Option<i64> {
        let s = time_str?.trim();
        if s.is_empty() {
            return None;
        }

        // 统一符号规范化: "2024:05:10 14:20:30.450" 或 "2024-05-10T14:20:30"
        let norm_s = s.replace('/', "-").replace('T', " ");
        let mut chars: Vec<char> = norm_s.chars().collect();
        if chars.len() >= 10 && chars[4] == ':' && chars[7] == ':' {
            chars[4] = '-';
            chars[7] = '-';
        }
        let clean_s: String = chars.into_iter().collect();

        let mut dt = None;
        for fmt in &[
            "%Y-%m-%d %H:%M:%S%.f",
            "%Y-%m-%d %H:%M:%S",
        ] {
            if let Ok(parsed) = NaiveDateTime::parse_from_str(&clean_s, fmt) {
                dt = Some(parsed);
                break;
            }
        }

        let dt = dt?;
        let mut ms = dt.and_utc().timestamp_millis();

        // 若原时间字符串不含小数点，但 EXIF 额外提取到 sub_sec_time
        if !clean_s.contains('.') {
            if let Some(sub) = sub_sec {
                let sub_clean = sub.trim().trim_start_matches('.');
                if !sub_clean.is_empty() {
                    let sub_val: Option<i64> = match sub_clean.len() {
                        1 => sub_clean.parse::<i64>().ok().map(|v| v * 100),
                        2 => sub_clean.parse::<i64>().ok().map(|v| v * 10),
                        3 => sub_clean.parse::<i64>().ok(),
                        _ => sub_clean.get(..3).and_then(|s| s.parse::<i64>().ok()),
                    };
                    if let Some(extra_ms) = sub_val {
                        ms += extra_ms;
                    }
                }
            }
        }

        Some(ms)
    }

    /// 从照片文件名末尾提取数字序列号 (如 "DSC00421" -> 421, "IMG_9999" -> 9999)
    pub fn extract_numeric_seq(name: &str) -> Option<u64> {
        let mut num_str = String::new();
        for ch in name.chars().rev() {
            if ch.is_ascii_digit() {
                num_str.push(ch);
            } else if !num_str.is_empty() {
                break;
            }
        }
        if num_str.is_empty() {
            None
        } else {
            let reversed: String = num_str.chars().rev().collect();
            reversed.parse::<u64>().ok()
        }
    }

    /// 执行连拍聚类算法
    ///
    /// 核心逻辑:
    /// 1. 按物理目录隔离照片，防止不同目录的照片被错误聚类
    /// 2. 提取每张照片的绝对时间戳 (毫秒) 与文件名末尾序列号
    /// 3. 毫秒级时间轴排序
    /// 4. 连续快门判定：两张照片间隔 <= threshold_ms (如 500ms)，或在同 1 秒内序列号连续递增 (+1)
    /// 5. 过滤数量 >= min_burst_count 的聚合组，赋予 burst_group_id
    /// 6. 自动为每组评估推选 Best Shot (综合考虑用户既有星级、留用旗标、文件体积/边缘细节)
    pub fn cluster_photos(
        photos: &mut [PhotoGroup],
        config: &BurstConfig,
    ) -> Vec<BurstCluster> {
        if photos.len() < config.min_burst_count {
            return Vec::new();
        }

        // 清理旧的连拍标记
        for p in photos.iter_mut() {
            p.burst_group_id = None;
            p.is_burst_best = false;
        }

        // 按目录将下标分类隔离
        let mut dir_to_indices: HashMap<String, Vec<usize>> = HashMap::new();
        for (idx, p) in photos.iter().enumerate() {
            let dir_key = p.folder_path.to_string_lossy().to_string();
            dir_to_indices.entry(dir_key).or_default().push(idx);
        }

        let mut clusters = Vec::new();
        let mut global_cluster_idx = 1usize;

        for (_dir, indices) in dir_to_indices {
            if indices.len() < config.min_burst_count {
                continue;
            }

            // 预提取元数据并排序
            let mut metas: Vec<ItemMeta> = indices
                .into_iter()
                .map(|idx| {
                    let p = &photos[idx];
                    let sub_sec = p.exif.as_ref().and_then(|e| e.sub_sec_time.as_deref());
                    let time_ms = Self::parse_timestamp_ms(p.capture_time.as_deref(), sub_sec);
                    let seq_num = Self::extract_numeric_seq(&p.base_name);
                    ItemMeta {
                        idx,
                        time_ms,
                        seq_num,
                        capture_time: p.capture_time.clone(),
                    }
                })
                .collect();

            // 排序规则: 优先按毫秒时间戳升序，无时间戳则按文件名序列号升序，最后按原名字典序
            metas.sort_by(|a, b| {
                match (a.time_ms, b.time_ms) {
                    (Some(t1), Some(t2)) => {
                        let c = t1.cmp(&t2);
                        if c != Ordering::Equal {
                            return c;
                        }
                    }
                    (Some(_), None) => return Ordering::Less,
                    (None, Some(_)) => return Ordering::Greater,
                    (None, None) => {}
                }
                match (a.seq_num, b.seq_num) {
                    (Some(s1), Some(s2)) => {
                        let c = s1.cmp(&s2);
                        if c != Ordering::Equal {
                            return c;
                        }
                    }
                    _ => {}
                }
                photos[a.idx].base_name.cmp(&photos[b.idx].base_name)
            });

            // 滑动窗口聚类
            let mut current_cluster: Vec<ItemMeta> = Vec::new();

            for item in metas {
                if current_cluster.is_empty() {
                    current_cluster.push(item);
                    continue;
                }

                let last = current_cluster.last().unwrap();
                let is_burst = match (last.time_ms, item.time_ms) {
                    (Some(t1), Some(t2)) => {
                        let diff = (t2 - t1).abs() as u64;
                        if diff <= config.threshold_ms {
                            true
                        } else if diff <= 1000 {
                            // 相机未写微秒时，若同秒或相差1秒且序列号相邻 (+1)
                            matches!((last.seq_num, item.seq_num), (Some(s1), Some(s2)) if s2 == s1 + 1)
                        } else {
                            false
                        }
                    }
                    _ => {
                        // 无时间戳纯靠文件名连续号
                        matches!((last.seq_num, item.seq_num), (Some(s1), Some(s2)) if s2 == s1 + 1)
                    }
                };

                if is_burst {
                    current_cluster.push(item);
                } else {
                    if current_cluster.len() >= config.min_burst_count {
                        let cluster = Self::finalize_cluster(
                            &current_cluster,
                            photos,
                            global_cluster_idx,
                            config.auto_flag_best,
                        );
                        clusters.push(cluster);
                        global_cluster_idx += 1;
                    }
                    current_cluster.clear();
                    current_cluster.push(item);
                }
            }

            if current_cluster.len() >= config.min_burst_count {
                let cluster = Self::finalize_cluster(
                    &current_cluster,
                    photos,
                    global_cluster_idx,
                    config.auto_flag_best,
                );
                clusters.push(cluster);
                global_cluster_idx += 1;
            }
        }

        clusters
    }

    fn finalize_cluster(
        cluster_items: &[ItemMeta],
        photos: &mut [PhotoGroup],
        cluster_num: usize,
        auto_flag_best: bool,
    ) -> BurstCluster {
        let group_id = format!("burst_{:03}", cluster_num);
        let photo_ids: Vec<String> = cluster_items
            .iter()
            .map(|item| photos[item.idx].id.clone())
            .collect();
        let count = photo_ids.len();

        let start_time = cluster_items.first().and_then(|i| i.capture_time.clone());
        let end_time = cluster_items.last().and_then(|i| i.capture_time.clone());

        let time_span_ms = match (
            cluster_items.first().and_then(|i| i.time_ms),
            cluster_items.last().and_then(|i| i.time_ms),
        ) {
            (Some(s), Some(e)) if e >= s => (e - s) as u64,
            _ => 0,
        };

        // 评选 Best Shot:
        // 1. 最高 rating (若用户已打标)
        // 2. flag == 1 (Pick)
        // 3. 最大文件体积 (高频细节多)
        // 4. 位于连拍正中间帧 (通常是主体姿态最舒展阶段)
        let mut best_idx_in_photos = cluster_items[count / 2].idx;
        let mut best_score = -1i64;

        for (order_in_group, item) in cluster_items.iter().enumerate() {
            let p = &photos[item.idx];
            let mut score = 0i64;
            score += (p.rating as i64) * 1_000_000_000;
            if p.flag == 1 {
                score += 500_000_000;
            } else if p.flag == -1 {
                score -= 500_000_000;
            }
            score += p.file_size_bytes as i64;
            // 距离中心越近给予轻微加权 (若文件体积近乎一致，优先取中心帧)
            let dist_from_center = (order_in_group as i64 - (count as i64 / 2)).abs();
            score -= dist_from_center * 200;

            if score > best_score {
                best_score = score;
                best_idx_in_photos = item.idx;
            }
        }

        let best_photo_id = Some(photos[best_idx_in_photos].id.clone());

        // 写回 PhotoGroup 实体
        for item in cluster_items.iter() {
            let is_best = item.idx == best_idx_in_photos;
            let p = &mut photos[item.idx];
            p.burst_group_id = Some(group_id.clone());
            p.is_burst_best = is_best;
            if is_best && auto_flag_best && p.flag == 0 {
                p.flag = 1;
            }
        }

        BurstCluster {
            group_id,
            photo_ids,
            count,
            best_photo_id,
            start_time,
            end_time,
            time_span_ms,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn make_test_photo(id: &str, name: &str, time: &str, sub: Option<&str>, size: u64) -> PhotoGroup {
        let mut p = PhotoGroup::new(id.to_string(), name.to_string(), PathBuf::from("C:/photos"));
        p.capture_time = Some(time.to_string());
        p.file_size_bytes = size;
        if let Some(s) = sub {
            let mut exif = crate::models::ExifSummary::default();
            exif.sub_sec_time = Some(s.to_string());
            p.exif = Some(exif);
        }
        p
    }

    #[test]
    fn test_burst_clustering() {
        let mut photos = vec![
            make_test_photo("1", "DSC_0001", "2024-05-10 12:00:00", Some("100"), 5000),
            make_test_photo("2", "DSC_0002", "2024-05-10 12:00:00", Some("250"), 5500),
            make_test_photo("3", "DSC_0003", "2024-05-10 12:00:00", Some("400"), 6000),
            // 距离下一张 5 秒，应该断开
            make_test_photo("4", "DSC_0004", "2024-05-10 12:00:05", Some("000"), 5200),
            make_test_photo("5", "DSC_0005", "2024-05-10 12:00:05", Some("200"), 5300),
        ];

        let config = BurstConfig {
            threshold_ms: 300,
            min_burst_count: 2,
            auto_flag_best: true,
        };

        let clusters = BurstDetector::cluster_photos(&mut photos, &config);
        assert_eq!(clusters.len(), 2);
        assert_eq!(clusters[0].count, 3);
        assert_eq!(clusters[1].count, 2);
        assert_eq!(photos[2].is_burst_best, true); // size 6000 is biggest
        assert_eq!(photos[2].flag, 1); // auto flagged as Pick
    }
}
