use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// 标准网络/通用图像格式后缀集合 (小写)
pub const STD_EXTENSIONS: &[&str] = &["jpg", "jpeg", "png", "heic", "heif", "webp", "tif", "tiff"];

/// 主流相机厂商 RAW 格式后缀集合 (小写)
pub const RAW_EXTENSIONS: &[&str] = &[
    "arw", // Sony
    "cr2", "cr3", // Canon
    "nef", "nrw", // Nikon
    "raf", // Fujifilm
    "rw2", // Panasonic
    "orf", // Olympus / OM System
    "pef", "dng", // Pentax / Leica / DJI / Apple ProRAW / Universal
    "3fr", // Hasselblad
    "iiq", // Phase One
];

/// 主流相机与通用视频格式后缀集合 (小写)
pub const VIDEO_EXTENSIONS: &[&str] = &[
    "mp4", "mov", "m4v", "mkv", "avi", "webm", "mts", "m2ts",
];

/// 归一化物理路径字符串 (跨平台兼容 Windows 反斜杠与 Linux 正斜杠)
pub fn normalize_path_str(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

/// 相机 EXIF 核心参数快速摘要
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct ExifSummary {
    pub camera_make: Option<String>,
    pub camera_model: Option<String>,
    pub lens_model: Option<String>,
    pub focal_length: Option<String>,
    pub focal_length_35mm: Option<String>,
    pub aperture: Option<String>,
    pub shutter_speed: Option<String>,
    pub iso: Option<u32>,
    pub exposure_compensation: Option<String>,
    pub capture_time: Option<String>,
    #[serde(default)]
    pub sub_sec_time: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub orientation: Option<u32>,
}

/// 聚合逻辑照片实体 (JPG + RAW + XMP 绑定)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PhotoGroup {
    /// 唯一散列标识 (UUID)
    pub id: String,
    /// 文件名主体 (无后缀，如 DSC0001)
    pub base_name: String,
    /// 所在物理目录绝对路径
    pub folder_path: PathBuf,
    /// 标准图像路径 (JPG / PNG / HEIC / WEBP)
    pub std_image: Option<PathBuf>,
    /// 相机原始 RAW 格式路径 (CR2 / CR3 / ARW / NEF / RAF / DNG)
    pub raw_image: Option<PathBuf>,
    /// 视频文件路径 (MP4 / MOV / M4V / WebM 等)
    #[serde(default)]
    pub video_file: Option<PathBuf>,
    /// 附属 XMP 元数据侧车文件路径
    pub xmp_sidecar: Option<PathBuf>,
    /// 星级打分: 0 - 5 星
    pub rating: u8,
    /// 旗标状态: 1 = Pick(留用), 0 = Unflagged(未标记), -1 = Reject(淘汰)
    pub flag: i8,
    /// 色标: "none", "red", "yellow", "green", "blue", "purple"
    pub color_label: String,
    /// 关键字与生态标签 (例如: ["红嘴相思鸟", "Leiothrix lutea", "动物|鸟类|红嘴相思鸟"])
    #[serde(default)]
    pub tags: Vec<String>,
    /// 连拍动作组 ID (如 "burst_001")
    #[serde(default)]
    pub burst_group_id: Option<String>,
    /// 是否为当前连拍动作组内最佳推荐帧 (Best Shot)
    #[serde(default)]
    pub is_burst_best: bool,
    /// 图像感知哈希 64-bit dHash (如 "a1b2c3d4e5f60718")
    #[serde(default)]
    pub phash: Option<String>,
    /// 同主体/场景聚类 ID (如 "subj_001")
    #[serde(default)]
    pub subject_group_id: Option<String>,
    /// 是否为当前同主体组内最佳推荐帧 (Best Shot)
    #[serde(default)]
    pub is_subject_best: bool,
    /// 是否为严格重复或极度雷同帧 (Duplicate)
    #[serde(default)]
    pub is_duplicate: bool,
    /// 若为重复帧，指向的主参考照片 ID
    #[serde(default)]
    pub duplicate_of: Option<String>,
    /// NPU/神经网络综合焦点与美学选优得分 (0 - 100)
    #[serde(default)]
    pub neural_score: Option<u8>,
    /// 主体中心区域高频锐度得分 (0.0 - 100.0)
    #[serde(default)]
    pub sharpness_score: Option<f32>,
    /// 神经网络自动识别的拍摄场景/主体类别标签
    #[serde(default)]
    pub scene_tag: Option<String>,
    /// 主文件字节数 (用于统计与安全校验)
    pub file_size_bytes: u64,
    /// 拍摄时间戳 ISO 8601
    pub capture_time: Option<String>,
    /// EXIF 核心四要素与机身镜头摘要
    pub exif: Option<ExifSummary>,
}

impl PhotoGroup {
    /// 创建新的空照片实体
    pub fn new(id: String, base_name: String, folder_path: PathBuf) -> Self {
        Self {
            id,
            base_name,
            folder_path,
            std_image: None,
            raw_image: None,
            video_file: None,
            xmp_sidecar: None,
            rating: 0,
            flag: 0,
            color_label: "none".to_string(),
            tags: Vec::new(),
            burst_group_id: None,
            is_burst_best: false,
            phash: None,
            subject_group_id: None,
            is_subject_best: false,
            is_duplicate: false,
            duplicate_of: None,
            neural_score: None,
            sharpness_score: None,
            scene_tag: None,
            file_size_bytes: 0,
            capture_time: None,
            exif: None,
        }
    }

    /// 获取该照片在本地数据库中的唯一指纹签名
    /// 由 (归一化目录路径 + 归一化大写文件名主体) 构成，跨平台绝对唯一且拔卡重插 100% 匹配
    pub fn signature(&self) -> String {
        let norm_folder = normalize_path_str(&self.folder_path);
        format!("{}:{}", norm_folder, self.base_name.to_uppercase())
    }

    /// 是否为视频文件
    #[inline]
    pub fn is_video(&self) -> bool {
        self.video_file.is_some()
    }

    /// 是否同时拥有 JPG 和 RAW 文件 (双格式同名联动)
    #[inline]
    pub fn is_raw_plus_jpg(&self) -> bool {
        self.std_image.is_some() && self.raw_image.is_some()
    }

    /// 是否仅包含 RAW 格式
    #[inline]
    pub fn is_raw_only(&self) -> bool {
        self.raw_image.is_some() && self.std_image.is_none()
    }

    /// 是否仅包含标准图像 (JPG / PNG)
    #[inline]
    pub fn is_std_only(&self) -> bool {
        self.std_image.is_some() && self.raw_image.is_none()
    }

    /// 获取界面显示的类型标签
    pub fn display_badge_label(&self) -> &'static str {
        if self.is_video() {
            "VIDEO"
        } else if self.is_raw_plus_jpg() {
            "RAW+JPG"
        } else if self.is_raw_only() {
            "RAW"
        } else {
            "JPG"
        }
    }

    /// 首选图像路径：优先读取 std_image，否则读取 raw_image
    pub fn primary_image_path(&self) -> Option<&PathBuf> {
        self.std_image.as_ref().or(self.raw_image.as_ref())
    }

    /// 检查指定路径后缀是否为标准图像
    pub fn is_std_ext(ext: &str) -> bool {
        STD_EXTENSIONS.iter().any(|&e| e.eq_ignore_ascii_case(ext))
    }

    /// 检查指定路径后缀是否为 RAW 格式
    pub fn is_raw_ext(ext: &str) -> bool {
        RAW_EXTENSIONS.iter().any(|&e| e.eq_ignore_ascii_case(ext))
    }

    /// 检查指定路径后缀是否为视频格式
    pub fn is_video_ext(ext: &str) -> bool {
        VIDEO_EXTENSIONS.iter().any(|&e| e.eq_ignore_ascii_case(ext))
    }

    /// 检查指定路径后缀是否为 XMP 侧车文件
    pub fn is_xmp_ext(ext: &str) -> bool {
        ext.eq_ignore_ascii_case("xmp")
    }

    /// 匹配过滤条件
    pub fn matches_filter(&self, query: &FilterQuery) -> bool {
        if self.rating < query.min_rating {
            return false;
        }

        if !query.allowed_flags.is_empty() && !query.allowed_flags.contains(&self.flag) {
            return false;
        }

        if !query.allowed_colors.is_empty()
            && !query.allowed_colors.iter().any(|c| c == &self.color_label)
        {
            return false;
        }

        if query.raw_plus_jpg_only && !self.is_raw_plus_jpg() {
            return false;
        }

        if query.video_only && !self.is_video() {
            return false;
        }

        if query.photo_only && self.is_video() {
            return false;
        }

        match query.smart_collection.as_deref() {
            Some("picked") => self.flag == 1,
            Some("rejected") => self.flag == -1,
            Some("unflagged") => self.flag == 0,
            Some("five_star") => self.rating == 5,
            Some("videos") => self.is_video(),
            _ => true,
        }
    }
}

/// 扫描统计结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanSummary {
    pub folder_path: String,
    pub total_groups: usize,
    pub raw_plus_jpg_count: usize,
    pub raw_only_count: usize,
    pub std_only_count: usize,
    #[serde(default)]
    pub video_count: usize,
    pub scan_elapsed_ms: u64,
    pub groups: Vec<PhotoGroup>,
}

/// 过滤查询请求体
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FilterQuery {
    pub min_rating: u8,
    pub allowed_flags: Vec<i8>,
    pub allowed_colors: Vec<String>,
    pub raw_plus_jpg_only: bool,
    #[serde(default)]
    pub video_only: bool,
    #[serde(default)]
    pub photo_only: bool,
    pub smart_collection: Option<String>,
}

/// 导出操作类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExportOp {
    #[serde(rename = "copy")]
    Copy,
    #[serde(rename = "move")]
    Move,
}

/// 导出文件策略
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExportStrategy {
    #[serde(rename = "raw_only")]
    RawOnly, // 仅导出同名 RAW
    #[serde(rename = "both")]
    Both, // 同时导出 RAW+JPG
    #[serde(rename = "std_only")]
    StdOnly, // 仅导出 JPG/PNG
}

/// 类 LrC 导出配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportConfig {
    pub target_dir: PathBuf,
    pub operation: ExportOp,
    pub export_strategy: ExportStrategy,
    pub smart_fallback: bool,
    pub split_folders: bool,
    pub generate_xmp: bool,
    pub rename_pattern: Option<String>,
    pub items: Vec<PhotoGroup>,
}

/// 导出实时进度与结果汇报
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportProgress {
    pub total: usize,
    pub current: usize,
    pub current_file: String,
    pub success_count: usize,
    pub fail_count: usize,
    pub errors: Vec<String>,
    pub is_finished: bool,
}

/// 本地 SQLite 存储的照片元数据记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhotoMetadataRecord {
    pub file_signature: String,
    pub base_name: String,
    pub folder_path: String,
    pub rating: u8,
    pub flag: i8,
    pub color_label: String,
    #[serde(default)]
    pub tags: Vec<String>,
    pub updated_at: String,
}

/// 工作区/目录历史记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceRecord {
    pub id: String,
    pub folder_path: String,
    pub volume_uuid: Option<String>,
    pub last_opened_at: String,
}

/// 连拍聚合分组配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BurstConfig {
    /// 判定为连拍的最大时间间隔（毫秒），默认 500ms
    #[serde(default = "default_burst_threshold_ms")]
    pub threshold_ms: u64,
    /// 构成连拍组的最少照片数量，默认 2 张
    #[serde(default = "default_min_burst_count")]
    pub min_burst_count: usize,
    /// 是否自动将每组的 Best Shot 推荐标记为 Pick (flag = 1)
    #[serde(default)]
    pub auto_flag_best: bool,
    /// 算力驱动方式: "auto" | "npu" | "gpu" | "cpu_ai" | "cpu_classic"
    #[serde(default)]
    pub compute_backend: Option<String>,
}

impl Default for BurstConfig {
    fn default() -> Self {
        Self {
            threshold_ms: 500,
            min_burst_count: 2,
            auto_flag_best: false,
            compute_backend: None,
        }
    }
}

fn default_burst_threshold_ms() -> u64 {
    500
}

fn default_min_burst_count() -> usize {
    2
}

/// 识别出的单组连拍聚合实体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BurstCluster {
    pub group_id: String,
    pub photo_ids: Vec<String>,
    pub count: usize,
    pub best_photo_id: Option<String>,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub time_span_ms: u64,
}

/// 严格去重与同主体聚类配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimilarityConfig {
    /// 判定为严格重复的 dHash 最大海明距离（默认 3，即 64 位中最多 3 位差异）
    #[serde(default = "default_dedup_threshold")]
    pub dedup_threshold: u32,
    /// 判定为同主体的 dHash 最大海明距离（默认 12，约 80%+ 视觉相似度）
    #[serde(default = "default_subject_threshold")]
    pub subject_threshold: u32,
    /// 同主体最大时间窗口（秒，默认 60s 内同场景同主体聚为一组）
    #[serde(default = "default_time_window_secs")]
    pub time_window_secs: u64,
    /// 构成组的最少照片数，默认 2 张
    #[serde(default = "default_min_group_count")]
    pub min_group_count: usize,
    /// 自动将严格重复的次选帧标记为淘汰 (Reject, flag = -1)
    #[serde(default)]
    pub auto_reject_duplicates: bool,
    /// 自动将同主体最佳推荐帧标记为留用 (Pick, flag = 1)
    #[serde(default)]
    pub auto_flag_best: bool,
    /// 算力驱动方式: "auto" | "npu" | "gpu" | "cpu_ai" | "cpu_classic"
    #[serde(default)]
    pub compute_backend: Option<String>,
    /// 是否将神经网络识别的场景/主体类别自动写入照片标签
    #[serde(default)]
    pub auto_tag_scene: bool,
}

impl Default for SimilarityConfig {
    fn default() -> Self {
        Self {
            dedup_threshold: 3,
            subject_threshold: 12,
            time_window_secs: 60,
            min_group_count: 2,
            auto_reject_duplicates: false,
            auto_flag_best: false,
            compute_backend: None,
            auto_tag_scene: false,
        }
    }
}

fn default_dedup_threshold() -> u32 {
    3
}

fn default_subject_threshold() -> u32 {
    12
}

fn default_time_window_secs() -> u64 {
    60
}

fn default_min_group_count() -> usize {
    2
}

/// 同主体/场景聚类与去重结果实体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubjectCluster {
    pub group_id: String,
    pub cluster_type: String, // "subject" | "duplicate"
    pub photo_ids: Vec<String>,
    pub count: usize,
    pub best_photo_id: Option<String>,
    pub duplicate_ids: Vec<String>,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub time_span_secs: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_video_extension_recognition() {
        assert!(PhotoGroup::is_video_ext("mp4"));
        assert!(PhotoGroup::is_video_ext("MP4"));
        assert!(PhotoGroup::is_video_ext("mov"));
        assert!(PhotoGroup::is_video_ext("MOV"));
        assert!(PhotoGroup::is_video_ext("m4v"));
        assert!(PhotoGroup::is_video_ext("mkv"));
        assert!(PhotoGroup::is_video_ext("mts"));
        assert!(PhotoGroup::is_video_ext("m2ts"));
        assert!(!PhotoGroup::is_video_ext("jpg"));
        assert!(!PhotoGroup::is_video_ext("arw"));
        assert!(!PhotoGroup::is_video_ext("cr3"));
    }

    #[test]
    fn test_video_group_and_filtering() {
        let mut group = PhotoGroup::new(
            "C0001".to_string(),
            "C0001".to_string(),
            PathBuf::from("D:/DCIM/100MSDCF"),
        );
        group.video_file = Some(PathBuf::from("D:/DCIM/100MSDCF/C0001.MP4"));
        assert!(group.is_video());
        assert_eq!(group.display_badge_label(), "VIDEO");

        let mut filter = FilterQuery {
            min_rating: 0,
            allowed_flags: vec![],
            allowed_colors: vec![],
            raw_plus_jpg_only: false,
            video_only: true,
            photo_only: false,
            smart_collection: None,
        };
        assert!(group.matches_filter(&filter));

        filter.video_only = false;
        filter.photo_only = true;
        assert!(!group.matches_filter(&filter));

        filter.photo_only = false;
        filter.smart_collection = Some("videos".to_string());
        assert!(group.matches_filter(&filter));

        filter.smart_collection = Some("picked".to_string());
        assert!(!group.matches_filter(&filter));
    }
}
