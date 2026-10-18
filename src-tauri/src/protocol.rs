use std::borrow::Cow;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use tauri::http::{header, Method, Request, Response, StatusCode};

use crate::db::Database;
use crate::decoder::ImageDecoder;

/// 注册于 Tauri 2.0 的自定义二进制零拷贝协议名称
pub const CUSTOM_PROTOCOL_SCHEME: &str = "photoselect";
static RAW_PREVIEW_LOCK: Mutex<()> = Mutex::new(());

/// 规范化从 URL / Query 中解析出的文件路径
fn normalize_file_path(raw: &str) -> String {
    let mut s = raw.to_string();
    #[cfg(windows)]
    {
        // 针对 Windows 驱动器盘符格式（如 /C: 或 //C: 或 \C: 或 /C|）
        let trimmed = s.trim_start_matches(|c| c == '/' || c == '\\');
        if trimmed.len() >= 2 {
            let b = trimmed.as_bytes();
            if (b[0] as char).is_ascii_alphabetic() && (b[1] == b':' || b[1] == b'|') {
                let mut res = trimmed.to_string();
                if res.as_bytes()[1] == b'|' {
                    res.replace_range(1..2, ":");
                }
                s = res;
            }
        }
    }
    s
}

fn is_raw_path(path: &Path) -> bool {
    matches!(
        path.extension()
            .and_then(|value| value.to_str())
            .unwrap_or_default()
            .to_ascii_lowercase()
            .as_str(),
        "arw"
            | "cr2"
            | "cr3"
            | "nef"
            | "nrw"
            | "raf"
            | "rw2"
            | "orf"
            | "pef"
            | "dng"
            | "3fr"
            | "iiq"
    )
}

/// 处理 photoselect:// (或 Windows 下 http://photoselect.localhost/) 协议的二进制图片请求
/// 典型 URL 形态：
/// - http://photoselect.localhost/F%3A%5CPhotos%2FDSC0001.JPG (Windows 全尺寸预览)
/// - http://photoselect.localhost/F%3A%5CPhotos%2FDSC0001.JPG?type=thumb (高速轻量缩略图)
pub fn handle_protocol_request(request: &Request<Vec<u8>>) -> Response<Cow<'static, [u8]>> {
    // 响应 CORS 预检请求
    if request.method() == Method::OPTIONS {
        return Response::builder()
            .status(StatusCode::NO_CONTENT)
            .header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
            .header(header::ACCESS_CONTROL_ALLOW_METHODS, "GET, HEAD, OPTIONS")
            .header(header::ACCESS_CONTROL_ALLOW_HEADERS, "*")
            .body(Cow::Borrowed(b"".as_slice()))
            .unwrap();
    }

    let uri = request.uri();
    let query = uri.query().unwrap_or("");
    let is_thumb =
        query.contains("type=thumb") || query.contains("thumb=1") || uri.path().contains("/thumb");

    let mut file_path_str = None;

    // 1. 尝试从 Query 参数中解析 (?path=...)
    if !query.is_empty() {
        for param in query.split('&') {
            let mut parts = param.splitn(2, '=');
            let key = parts.next().unwrap_or("");
            let val = parts.next().unwrap_or("");
            if key.eq_ignore_ascii_case("path") {
                if let Ok(decoded) = urlencoding::decode(val) {
                    let s = normalize_file_path(&decoded);
                    if !s.is_empty() {
                        file_path_str = Some(s);
                        break;
                    }
                }
            }
        }
    }

    // 2. 如果 Query 没有 path 参数，从 URI Path 中获取 (convertFileSrc 标准形态)
    if file_path_str.is_none() {
        let raw_path = uri.path();
        if !raw_path.is_empty() && raw_path != "/" && raw_path != "/preview" && raw_path != "/thumb"
        {
            let path_part = if let Some(stripped) = raw_path.strip_prefix("/preview/") {
                stripped
            } else if let Some(stripped) = raw_path.strip_prefix("/preview") {
                stripped
            } else if let Some(stripped) = raw_path.strip_prefix("/thumb/") {
                stripped
            } else if let Some(stripped) = raw_path.strip_prefix("/thumb") {
                stripped
            } else {
                raw_path
            };

            if let Ok(decoded) = urlencoding::decode(path_part) {
                let s = normalize_file_path(&decoded);
                if !s.is_empty() && s != "/" {
                    file_path_str = Some(s);
                }
            }
        }
    }

    let raw_path = match file_path_str {
        Some(p) if !p.is_empty() => p,
        _ => {
            return Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
                .header(header::CONTENT_TYPE, "text/plain; charset=utf-8")
                .body(Cow::Borrowed(
                    b"Missing or invalid 'path' parameter".as_slice(),
                ))
                .unwrap();
        }
    };

    let path = PathBuf::from(&raw_path);
    if !path.exists() || !path.is_file() {
        return Response::builder()
            .status(StatusCode::NOT_FOUND)
            .header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
            .header(header::CONTENT_TYPE, "text/plain; charset=utf-8")
            .body(Cow::Owned(
                format!("Target file does not exist: {}", raw_path).into_bytes(),
            ))
            .unwrap();
    }

    // =========================================================================
    // 缩略图模式 (胶片条与网格专用，优先从 SSD 缓存与内嵌高速小图命中)
    // =========================================================================
    if is_thumb {
        if let Ok(app_dir) = Database::resolve_app_data_dir() {
            let cache_dir = app_dir.join("cache").join("thumbs");
            let file_len = path.metadata().map(|m| m.len()).unwrap_or(0);
            let mtime_nanos = path
                .metadata()
                .and_then(|m| m.modified())
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_nanos())
                .unwrap_or(0);

            let mut hasher = DefaultHasher::new();
            path.hash(&mut hasher);
            "thumb-v1".hash(&mut hasher);
            file_len.hash(&mut hasher);
            mtime_nanos.hash(&mut hasher);
            let cache_file = cache_dir.join(format!("{:016x}.jpg", hasher.finish()));

            // 1. 命中高速 SSD 磁盘缓存 (0.05ms)
            if cache_file.exists() {
                if let Ok(cached_bytes) = std::fs::read(&cache_file) {
                    return Response::builder()
                        .status(StatusCode::OK)
                        .header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
                        .header(header::ACCESS_CONTROL_ALLOW_METHODS, "GET, HEAD, OPTIONS")
                        .header(header::ACCESS_CONTROL_ALLOW_HEADERS, "*")
                        .header(header::CONTENT_TYPE, "image/jpeg")
                        .header(header::CACHE_CONTROL, "public, max-age=31536000, immutable")
                        .header(header::CONTENT_LENGTH, cached_bytes.len().to_string())
                        .body(Cow::Owned(cached_bytes))
                        .unwrap();
                }
            }

            // 2. 缓存未命中：提取嵌入式轻量级缩略图并写入缓存
            if let Ok(thumb_bytes) = ImageDecoder::extract_thumbnail_bytes(&path) {
                let _ = std::fs::create_dir_all(&cache_dir);
                let _ = std::fs::write(&cache_file, &thumb_bytes);

                return Response::builder()
                    .status(StatusCode::OK)
                    .header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
                    .header(header::ACCESS_CONTROL_ALLOW_METHODS, "GET, HEAD, OPTIONS")
                    .header(header::ACCESS_CONTROL_ALLOW_HEADERS, "*")
                    .header(header::CONTENT_TYPE, "image/jpeg")
                    .header(header::CACHE_CONTROL, "public, max-age=31536000, immutable")
                    .header(header::CONTENT_LENGTH, thumb_bytes.len().to_string())
                    .body(Cow::Owned(thumb_bytes))
                    .unwrap();
            }
        } else if let Ok(thumb_bytes) = ImageDecoder::extract_thumbnail_bytes(&path) {
            return Response::builder()
                .status(StatusCode::OK)
                .header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
                .header(header::ACCESS_CONTROL_ALLOW_METHODS, "GET, HEAD, OPTIONS")
                .header(header::ACCESS_CONTROL_ALLOW_HEADERS, "*")
                .header(header::CONTENT_TYPE, "image/jpeg")
                .header(header::CACHE_CONTROL, "public, max-age=31536000, immutable")
                .header(header::CONTENT_LENGTH, thumb_bytes.len().to_string())
                .body(Cow::Owned(thumb_bytes))
                .unwrap();
        }
    }

    // =========================================================================
    // 全尺寸单图视口预览模式。RAW 开发结果写入版本化磁盘缓存，避免每次
    // 进入同一张照片都重新解马赛克。缓存键包含路径、大小和修改时间。
    // =========================================================================
    let preview_cache = if is_raw_path(&path) {
        Database::resolve_app_data_dir()
            .ok()
            .map(|app_dir| {
                let file_len = path.metadata().map(|m| m.len()).unwrap_or(0);
                let mtime_nanos = path
                    .metadata()
                    .and_then(|m| m.modified())
                    .ok()
                    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                    .map(|d| d.as_nanos())
                    .unwrap_or(0);
                let mut hasher = DefaultHasher::new();
                path.hash(&mut hasher);
                "raw-srgb-v1".hash(&mut hasher);
                file_len.hash(&mut hasher);
                mtime_nanos.hash(&mut hasher);
                Some(
                    app_dir
                        .join("cache")
                        .join("previews")
                        .join(format!("{:016x}.jpg", hasher.finish())),
                )
            })
            .flatten()
    } else {
        None
    };

    if let Some(cache_file) = preview_cache.as_ref() {
        if let Ok(cached_bytes) = std::fs::read(cache_file) {
            return Response::builder()
                .status(StatusCode::OK)
                .header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
                .header(header::CONTENT_TYPE, "image/jpeg")
                .header(header::CACHE_CONTROL, "public, max-age=31536000, immutable")
                .header(header::CONTENT_LENGTH, cached_bytes.len().to_string())
                .body(Cow::Owned(cached_bytes))
                .unwrap();
        }
    }

    let _raw_preview_guard = if preview_cache.is_some() {
        Some(
            RAW_PREVIEW_LOCK
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner()),
        )
    } else {
        None
    };

    if let Some(cache_file) = preview_cache.as_ref() {
        if let Ok(cached_bytes) = std::fs::read(cache_file) {
            return Response::builder()
                .status(StatusCode::OK)
                .header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
                .header(header::CONTENT_TYPE, "image/jpeg")
                .header(header::CACHE_CONTROL, "private, no-cache, must-revalidate")
                .header(header::CONTENT_LENGTH, cached_bytes.len().to_string())
                .body(Cow::Owned(cached_bytes))
                .unwrap();
        }
    }

    let preview_result = ImageDecoder::extract_preview_bytes(&path);
    if let (Some(cache_file), Ok(bytes)) = (preview_cache, &preview_result) {
        if is_raw_path(&path) {
            if let Some(parent) = cache_file.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            let pending_file = cache_file.with_extension("jpg.pending");
            if std::fs::write(&pending_file, bytes).is_ok() {
                let _ = std::fs::rename(&pending_file, cache_file);
            }
        }
    }

    match preview_result {
        Ok(image_bytes) => {
            let content_type = detect_mime_type(&path);
            Response::builder()
                .status(StatusCode::OK)
                .header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
                .header(header::ACCESS_CONTROL_ALLOW_METHODS, "GET, HEAD, OPTIONS")
                .header(header::ACCESS_CONTROL_ALLOW_HEADERS, "*")
                .header(header::CONTENT_TYPE, content_type)
                .header(header::CACHE_CONTROL, "private, no-cache, must-revalidate")
                .header(header::CONTENT_LENGTH, image_bytes.len().to_string())
                .body(Cow::Owned(image_bytes))
                .unwrap()
        }
        Err(err) => Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
            .header(header::CONTENT_TYPE, "text/plain; charset=utf-8")
            .body(Cow::Owned(
                format!("Failed to extract preview stream: {:?}", err).into_bytes(),
            ))
            .unwrap(),
    }
}

/// 识别返回给前端的 MIME Content-Type
fn detect_mime_type(path: &Path) -> &'static str {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    match ext.as_str() {
        "png" => "image/png",
        "webp" => "image/webp",
        // 所有主流相机 RAW 提取出的内嵌预览图均为标准 JPEG 格式
        _ => "image/jpeg",
    }
}
