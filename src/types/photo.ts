/**
 * PhotoSelect Desktop - 核心数据结构接口定义
 */

export type FlagStatus = 1 | 0 | -1; // 1: Pick (留用), 0: Unflagged, -1: Reject (淘汰)
export type ColorLabel = "none" | "red" | "yellow" | "green" | "blue" | "purple";
export type ViewMode = "loupe" | "grid" | "compare";

export interface ExifSummary {
  camera_make?: string;
  camera_model?: string;
  lens_model?: string;
  focal_length?: string;
  focal_length_35mm?: string;
  aperture?: string;
  shutter_speed?: string;
  iso?: number;
  exposure_compensation?: string;
  capture_time?: string;
  sub_sec_time?: string;
  width?: number;
  height?: number;
  orientation?: number;
}

export interface PhotoGroup {
  id: string;                      // 唯一标识 (UUID / Hash)
  base_name: string;               // 文件名主体 (无后缀，如 DSC0001)
  folder_path: string;             // 所属父目录绝对路径
  std_image: string | null;        // JPG / PNG / HEIC / WEBP 绝对路径
  raw_image: string | null;        // CR2 / CR3 / ARW / NEF / RAF / DNG 等绝对路径
  xmp_sidecar: string | null;      // 对应同名 .xmp 文件绝对路径
  rating: number;                  // 0 - 5 星
  flag: FlagStatus;                // 1(留用), 0(未标记), -1(淘汰)
  color_label: ColorLabel;         // 色彩标签
  tags?: string[];                 // 关键字与生态/分类标签 (如 ["红嘴相思鸟", "动物|鸟类|红嘴相思鸟"])
  burst_group_id?: string | null;  // 连拍动作组 ID (如 "burst_001")
  is_burst_best?: boolean;         // 是否为连拍组推荐最佳帧 (Best Shot)
  phash?: string | null;           // 64-bit dHash 图像感知指纹
  subject_group_id?: string | null;// 同主体/场景聚类组 ID (如 "subj_001")
  is_subject_best?: boolean;       // 是否为同主体推荐最佳帧
  is_duplicate?: boolean;          // 是否为严格重复帧
  duplicate_of?: string | null;    // 重复所指向的主参考照片 ID
  file_size_bytes: number;         // 照片主文件大小
  capture_time: string | null;     // 拍摄时间戳 ISO 8601
  exif: ExifSummary | null;        // EXIF 核心元数据
}

export interface ScanResult {
  folder_path: string;
  total_groups: number;
  raw_plus_jpg_count: number;
  raw_only_count: number;
  std_only_count: number;
  scan_elapsed_ms: number;
  groups: PhotoGroup[];
}

export interface FilterState {
  min_rating: number;              // 0-5
  flags: FlagStatus[];             // 允许展示的旗标
  colors: ColorLabel[];            // 允许展示的色标
  raw_plus_jpg_only: boolean;      // 仅展示 JPG+RAW 联动文件
  smart_collection: "all" | "picked" | "rejected" | "unflagged" | "five_star";
}

export type ExportOp = "copy" | "move";
export type ExportStrategy = "raw_only" | "both" | "std_only";

export interface ExportConfig {
  target_dir: string;
  operation: ExportOp;
  export_strategy: ExportStrategy;
  smart_fallback: boolean;
  split_folders: boolean;
  generate_xmp: boolean;
  rename_pattern: string | null;
  items: PhotoGroup[];
}

export interface ExportProgress {
  total: number;
  current: number;
  current_file: string;
  success_count: number;
  fail_count: number;
  errors: string[];
  is_finished: boolean;
}

export interface WorkspaceRecord {
  id: string;
  folder_path: string;
  volume_uuid: string | null;
  last_opened_at: string;
}

export interface PhotoMetadataRecord {
  file_signature: string;
  base_name: string;
  folder_path: string;
  rating: number;
  flag: FlagStatus;
  color_label: ColorLabel;
  tags?: string[];
  updated_at: string;
}

export interface BurstConfig {
  threshold_ms?: number;
  min_burst_count?: number;
  auto_flag_best?: boolean;
}

export interface BurstCluster {
  group_id: string;
  photo_ids: string[];
  count: number;
  best_photo_id: string | null;
  start_time: string | null;
  end_time: string | null;
  time_span_ms: number;
}

export interface SimilarityConfig {
  dedup_threshold?: number;
  subject_threshold?: number;
  time_window_secs?: number;
  min_group_count?: number;
  auto_reject_duplicates?: boolean;
  auto_flag_best?: boolean;
}

export interface SubjectCluster {
  group_id: string;
  cluster_type: "subject" | "duplicate";
  photo_ids: string[];
  count: number;
  best_photo_id: string | null;
  duplicate_ids: string[];
  start_time: string | null;
  end_time: string | null;
  time_span_secs: number;
}

