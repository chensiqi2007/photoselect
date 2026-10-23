pub mod db;
pub mod ai_selection;
pub mod burst;
pub mod decoder;
pub mod exporter;
pub mod models;
pub mod protocol;
pub mod scanner;
pub mod similarity;

use std::path::{Path, PathBuf};
use tauri::{AppHandle, State};

use crate::db::Database;
use crate::decoder::ImageDecoder;
use crate::exporter::Exporter;
use crate::models::{ExifSummary, ExportConfig, ExportProgress, ScanSummary, WorkspaceRecord};
use crate::protocol::{handle_protocol_request, CUSTOM_PROTOCOL_SCHEME};
use crate::scanner::DirectoryScanner;

#[tauri::command]
fn get_ai_selection_settings(state: State<'_, AppState>) -> Result<ai_selection::AiSelectionSettings, String> {
    ai_selection::load_settings(&state.db).map_err(|error| format!("{error:#}"))
}

#[tauri::command]
fn save_ai_selection_settings(
    enabled: bool,
    base_url: String,
    model: String,
    api_key: Option<String>,
    clear_api_key: bool,
    state: State<'_, AppState>,
) -> Result<ai_selection::AiSelectionSettings, String> {
    ai_selection::save_settings(&state.db, enabled, &base_url, &model, api_key.as_deref(), clear_api_key)
        .map_err(|error| format!("{error:#}"))
}

#[tauri::command]
async fn test_ai_connection(state: State<'_, AppState>) -> Result<String, String> {
    ai_selection::test_connection(&state.db).await.map_err(|error| format!("{error:#}"))
}

#[tauri::command]
async fn start_ai_selection(
    photos: Vec<crate::models::PhotoGroup>,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    ai_selection::rank_photos(&state.db, &app, photos).await.map_err(|error| format!("{error:#}"))
}

/// 全局共享状态
pub struct AppState {
    pub db: Database,
}

// =========================================================================
// Tauri 2.0 Commands (前后端通信指令系统)
// 注意：在 lib.rs 内部使用非 pub 函数定义命令，避免宏生成同名内部导出冲突
// =========================================================================

/// 扫描指定目录并执行 JPG+RAW 智能配对，并自动加载本地已暂存的打标进度
#[tauri::command]
async fn scan_directory(
    path: String,
    recursive: Option<bool>,
    state: State<'_, AppState>,
) -> Result<ScanSummary, String> {
    let dir_path = PathBuf::from(&path);
    if !dir_path.exists() || !dir_path.is_dir() {
        return Err(format!("指定路径不存在或不是有效目录: {}", path));
    }

    let is_recursive = recursive.unwrap_or(false);
    let db = state.db.clone();

    // 在 Tokio 异步工作线程中运行扫描
    tokio::task::spawn_blocking(move || {
        let mut summary = DirectoryScanner::scan(&dir_path, is_recursive)
            .map_err(|e| format!("扫描目录失败: {}", e))?;

        // 1. 记录工作区至 SQLite
        let _ = db.record_workspace(&path, None);

        // 2. 从 SQLite 读取该目录下所有已保存的评级/旗标数据并进行签名对齐
        if let Ok(saved_meta) = db.get_metadata_for_folder(&path) {
            for group in &mut summary.groups {
                let sig = group.signature();
                if let Some(record) = saved_meta.get(&sig) {
                    group.rating = record.rating;
                    group.flag = record.flag;
                    group.color_label = record.color_label.clone();
                    group.tags = record.tags.clone();
                }
            }
        }

        Ok(summary)
    })
    .await
    .map_err(|e| format!("扫描任务执行失败: {}", e))?
}

/// 快速提取指定图片或 RAW 文件的 EXIF 摘要参数
#[tauri::command]
async fn get_file_exif(path: String) -> Result<Option<ExifSummary>, String> {
    let file_path = PathBuf::from(&path);
    if !file_path.exists() {
        return Err(format!("文件不存在: {}", path));
    }

    tokio::task::spawn_blocking(move || Ok(ImageDecoder::extract_exif_summary(&file_path)))
        .await
        .map_err(|e| format!("EXIF 任务失败: {}", e))?
}

/// 获取系统可用的物理磁盘与外置存储挂载点 (SD卡/TF卡/PSSD)
#[tauri::command]
fn check_mount_points() -> Result<Vec<String>, String> {
    let mut mounts = Vec::new();

    #[cfg(target_os = "windows")]
    {
        for drive in b'C'..=b'Z' {
            let drive_path = format!("{}:\\", drive as char);
            if Path::new(&drive_path).exists() {
                mounts.push(drive_path);
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        let user = std::env::var("USER").unwrap_or_default();
        let possible_roots = [
            format!("/run/media/{}", user),
            format!("/media/{}", user),
            "/mnt".to_string(),
        ];

        for root in &possible_roots {
            let p = Path::new(root);
            if p.is_dir() {
                if let Ok(entries) = std::fs::read_dir(p) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.is_dir() {
                            mounts.push(path.to_string_lossy().to_string());
                        }
                    }
                }
            }
        }
    }

    Ok(mounts)
}

/// 原子单独更新星级评分 (0-5)
#[tauri::command]
async fn update_photo_rating(
    signature: String,
    base_name: String,
    folder_path: String,
    rating: u8,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let db = state.db.clone();
    tokio::task::spawn_blocking(move || {
        db.update_rating(&signature, &base_name, &folder_path, rating)
            .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

/// 原子单独更新旗标状态 (Pick: 1, Unflag: 0, Reject: -1)
#[tauri::command]
async fn update_photo_flag(
    signature: String,
    base_name: String,
    folder_path: String,
    flag: i8,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let db = state.db.clone();
    tokio::task::spawn_blocking(move || {
        db.update_flag(&signature, &base_name, &folder_path, flag)
            .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

/// 原子单独更新色彩标签
#[tauri::command]
async fn update_photo_color_label(
    signature: String,
    base_name: String,
    folder_path: String,
    color_label: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let db = state.db.clone();
    tokio::task::spawn_blocking(move || {
        db.update_color_label(&signature, &base_name, &folder_path, &color_label)
            .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

/// 批量更新照片标记状态 (全量批量打标)
#[tauri::command]
async fn batch_update_photo_status(
    records: Vec<(String, String, String, u8, i8, String)>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let db = state.db.clone();
    tokio::task::spawn_blocking(move || {
        db.batch_upsert_metadata(&records)
            .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

/// 获取最近打开的工作区历史
#[tauri::command]
async fn get_recent_workspaces(state: State<'_, AppState>) -> Result<Vec<WorkspaceRecord>, String> {
    let db = state.db.clone();
    tokio::task::spawn_blocking(move || db.get_recent_workspaces().map_err(|e| e.to_string()))
        .await
        .map_err(|e| e.to_string())?
}

/// 执行类 LrC 快捷导出与 XMP 侧车文件生成
#[tauri::command]
async fn execute_export(
    config: ExportConfig,
    app_handle: AppHandle,
) -> Result<ExportProgress, String> {
    tokio::task::spawn_blocking(move || {
        tokio::runtime::Handle::current().block_on(async {
            Exporter::execute(&config, Some(&app_handle))
                .await
                .map_err(|e| format!("导出任务异常: {:#}", e))
        })
    })
    .await
    .map_err(|e| format!("导出工作线程异常: {}", e))?
}

/// 原子单独更新照片关键字与生态标签 (持久化至 SQLite)
#[tauri::command]
async fn update_photo_tags(
    signature: String,
    base_name: String,
    folder_path: String,
    tags: Vec<String>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let db = state.db.clone();
    tokio::task::spawn_blocking(move || {
        db.update_tags(&signature, &base_name, &folder_path, &tags)
            .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

/// 执行毫秒级高速连拍聚合算法 (根据时间戳与文件名序列精准分组并标定 Best Shot)
#[tauri::command]
fn cluster_burst_photos(
    mut photos: Vec<crate::models::PhotoGroup>,
    config: Option<crate::models::BurstConfig>,
) -> Result<(Vec<crate::models::PhotoGroup>, Vec<crate::models::BurstCluster>), String> {
    let cfg = config.unwrap_or_default();
    let clusters = crate::burst::BurstDetector::cluster_photos(&mut photos, &cfg);
    Ok((photos, clusters))
}

/// 为指定单张照片在其同目录下写入/更新行业标准 XMP 侧车文件 (支持生态层级与平铺关键字)
#[tauri::command]
async fn write_photo_xmp(
    photo: crate::models::PhotoGroup,
    extra_keywords: Option<Vec<String>>,
) -> Result<String, String> {
    tokio::task::spawn_blocking(move || {
        let xmp_path = Exporter::write_xmp_sidecar_for_photo(&photo, extra_keywords.as_deref())
            .map_err(|e| format!("写入 XMP 侧车文件失败: {:#}", e))?;
        Ok(xmp_path.to_string_lossy().to_string())
    })
    .await
    .map_err(|e| format!("XMP 写入任务异常: {}", e))?
}

/// 执行严格去重与同主体视觉聚类分析 (基于 64-bit dHash 与时序滑动窗口)
#[tauri::command]
async fn detect_similarity_and_duplicates(
    mut photos: Vec<crate::models::PhotoGroup>,
    config: Option<crate::models::SimilarityConfig>,
) -> Result<(Vec<crate::models::PhotoGroup>, Vec<crate::models::SubjectCluster>), String> {
    tokio::task::spawn_blocking(move || {
        let cfg = config.unwrap_or_default();
        let clusters = crate::similarity::SimilarityDetector::detect_duplicates_and_subjects(&mut photos, &cfg);
        Ok((photos, clusters))
    })
    .await
    .map_err(|e| format!("去重与相似度分析任务异常: {}", e))?
}

// =========================================================================
// Tauri 2.0 入口与生命周期构建器
// =========================================================================

pub fn run() {
    let db = Database::init().expect("本地 SQLite 持久化数据库初始化失败");
    let state = AppState { db };

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(state)
        // 注册 photoselect:// 自定义异步二进制流协议 (零 Base64 开销)
        .register_asynchronous_uri_scheme_protocol(
            CUSTOM_PROTOCOL_SCHEME,
            |_app, request, responder| {
                std::thread::spawn(move || {
                    let response = handle_protocol_request(&request);
                    responder.respond(response);
                });
            },
        )
        .invoke_handler(tauri::generate_handler![
            scan_directory,
            get_file_exif,
            check_mount_points,
            update_photo_rating,
            update_photo_flag,
            update_photo_color_label,
            update_photo_tags,
            batch_update_photo_status,
            get_recent_workspaces,
            execute_export,
            cluster_burst_photos,
            detect_similarity_and_duplicates,
            write_photo_xmp,
            get_ai_selection_settings,
            save_ai_selection_settings,
            test_ai_connection,
            start_ai_selection,
        ])
        .run(tauri::generate_context!())
        .expect("PhotoSelect Desktop 运行时启动异常");
}
