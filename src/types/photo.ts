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
  updated_at: string;
}
