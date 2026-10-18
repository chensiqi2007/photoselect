use anyhow::{bail, Context, Result};
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Emitter};

use crate::models::{ExportConfig, ExportOp, ExportProgress, ExportStrategy, PhotoGroup};

/// 类 Lightroom Classic 快捷导出与 XMP 侧车桥接引擎
pub struct Exporter;

impl Exporter {
    /// 执行完整导出流水线，带进度通知与防丢片校验
    pub async fn execute(
        config: &ExportConfig,
        app_handle: Option<&AppHandle>,
    ) -> Result<ExportProgress> {
        let total = config.items.len();
        let mut progress = ExportProgress {
            total,
            current: 0,
            current_file: String::new(),
            success_count: 0,
            fail_count: 0,
            errors: Vec::new(),
            is_finished: false,
        };

        if total == 0 {
            progress.is_finished = true;
            return Ok(progress);
        }

        // 1. 确保目标根目录存在
        fs::create_dir_all(&config.target_dir).with_context(|| {
            format!(
                "Failed to create target export directory: {:?}",
                config.target_dir
            )
        })?;

        // 2. 若选择隔离存放，创建 /RAW 与 /JPG 子文件夹
        let raw_target_dir = if config.split_folders {
            let p = config.target_dir.join("RAW");
            fs::create_dir_all(&p)?;
            p
        } else {
            config.target_dir.clone()
        };

        let std_target_dir = if config.split_folders {
            let p = config.target_dir.join("JPG");
            fs::create_dir_all(&p)?;
            p
        } else {
            config.target_dir.clone()
        };

        // 3. 逐项流式执行导出
        for (idx, item) in config.items.iter().enumerate() {
            progress.current = idx + 1;
            progress.current_file = item.base_name.clone();

            // 发送实时进度事件给前端
            if let Some(app) = app_handle {
                let _ = app.emit("export-progress", &progress);
            }

            match Self::export_single_item(item, idx + 1, config, &raw_target_dir, &std_target_dir)
            {
                Ok(_) => {
                    progress.success_count += 1;
                }
                Err(err) => {
                    progress.fail_count += 1;
                    progress
                        .errors
                        .push(format!("照片 [{}] 导出失败: {:#}", item.base_name, err));
                }
            }
        }

        progress.is_finished = true;
        progress.current_file = "导出完成".to_string();

        if let Some(app) = app_handle {
            let _ = app.emit("export-progress", &progress);
        }

        Ok(progress)
    }

    /// 导出单组照片 (处理同名 RAW、JPG 及 XMP 联动与重命名)
    fn export_single_item(
        item: &PhotoGroup,
        seq_num: usize,
        config: &ExportConfig,
        raw_target_dir: &Path,
        std_target_dir: &Path,
    ) -> Result<()> {
        // 计算目标文件名 (支持自定义模板重命名，同时赋予 RAW、JPG、XMP 相同的 base_name)
        let target_base_name =
            Self::generate_target_base_name(item, seq_num, config.rename_pattern.as_deref());

        let mut targets_to_transfer: Vec<(&PathBuf, PathBuf)> = Vec::with_capacity(3);

        // 根据策略确定待导出文件
        match config.export_strategy {
            ExportStrategy::RawOnly => {
                if let Some(ref raw_path) = item.raw_image {
                    let ext = raw_path
                        .extension()
                        .and_then(|e| e.to_str())
                        .unwrap_or("RAW");
                    let dst = raw_target_dir.join(format!("{}.{}", target_base_name, ext));
                    targets_to_transfer.push((raw_path, dst));
                } else if config.smart_fallback && item.std_image.is_some() {
                    // 智能回退：选仅导出 RAW 但相机当时只拍了 JPG，防止漏片
                    let std_path = item.std_image.as_ref().unwrap();
                    let ext = std_path
                        .extension()
                        .and_then(|e| e.to_str())
                        .unwrap_or("jpg");
                    let dst = std_target_dir.join(format!("{}.{}", target_base_name, ext));
                    targets_to_transfer.push((std_path, dst));
                }
            }
            ExportStrategy::Both => {
                if let Some(ref raw_path) = item.raw_image {
                    let ext = raw_path
                        .extension()
                        .and_then(|e| e.to_str())
                        .unwrap_or("RAW");
                    let dst = raw_target_dir.join(format!("{}.{}", target_base_name, ext));
                    targets_to_transfer.push((raw_path, dst));
                }
                if let Some(ref std_path) = item.std_image {
                    let ext = std_path
                        .extension()
                        .and_then(|e| e.to_str())
                        .unwrap_or("jpg");
                    let dst = std_target_dir.join(format!("{}.{}", target_base_name, ext));
                    targets_to_transfer.push((std_path, dst));
                }
            }
            ExportStrategy::StdOnly => {
                if let Some(ref std_path) = item.std_image {
                    let ext = std_path
                        .extension()
                        .and_then(|e| e.to_str())
                        .unwrap_or("jpg");
                    let dst = std_target_dir.join(format!("{}.{}", target_base_name, ext));
                    targets_to_transfer.push((std_path, dst));
                } else if config.smart_fallback && item.raw_image.is_some() {
                    let raw_path = item.raw_image.as_ref().unwrap();
                    let ext = raw_path
                        .extension()
                        .and_then(|e| e.to_str())
                        .unwrap_or("RAW");
                    let dst = raw_target_dir.join(format!("{}.{}", target_base_name, ext));
                    targets_to_transfer.push((raw_path, dst));
                }
            }
        }

        if targets_to_transfer.is_empty() {
            bail!("无可导出的目标文件 (既无 RAW 也无 JPG)");
        }

        // 执行安全传输 (Copy 或 Move)
        for (src_path, dst_path) in &targets_to_transfer {
            Self::transfer_file_safe(src_path, dst_path, config.operation)?;
        }

        // 生成 Adobe Lightroom Classic 兼容的 .xmp 侧车文件
        if config.generate_xmp {
            let xmp_content = Self::generate_xmp_content(item.rating, &item.color_label, item.flag);

            // 若导出了 RAW，XMP 与 RAW 存放在一起；否则与 JPG 放在一起
            let xmp_dest_dir =
                if item.raw_image.is_some() && config.export_strategy != ExportStrategy::StdOnly {
                    raw_target_dir
                } else {
                    std_target_dir
                };

            let xmp_path = xmp_dest_dir.join(format!("{}.xmp", target_base_name));
            let mut file = File::create(&xmp_path)
                .with_context(|| format!("Failed to create XMP file: {:?}", xmp_path))?;
            file.write_all(xmp_content.as_bytes())?;
            file.sync_all()?;
        }

        Ok(())
    }

    /// 安全文件流转机制 (防丢片设计)
    /// 在 Move 模式下，必须确认目标文件写入成功、字节大小一致并 sync_all 刷入物理介质后，才允许删除源文件
    fn transfer_file_safe(src: &Path, dst: &Path, op: ExportOp) -> Result<()> {
        let src_len = fs::metadata(src)
            .with_context(|| format!("Source file not found: {:?}", src))?
            .len();

        match op {
            ExportOp::Copy => {
                let copied_bytes = fs::copy(src, dst)
                    .with_context(|| format!("Failed to copy {:?} to {:?}", src, dst))?;
                if copied_bytes != src_len {
                    bail!(
                        "Copy byte mismatch: expected {}, copied {}",
                        src_len,
                        copied_bytes
                    );
                }
            }
            ExportOp::Move => {
                // 先安全写入目标文件
                let copied_bytes = fs::copy(src, dst)
                    .with_context(|| format!("Failed to copy for move {:?} to {:?}", src, dst))?;
                if copied_bytes != src_len {
                    bail!(
                        "Move aborted: copied size {} != source size {}",
                        copied_bytes,
                        src_len
                    );
                }

                // 强制将目标文件刷入物理磁盘
                let target_file = File::open(dst)?;
                target_file.sync_all()?;
                drop(target_file);

                // 确认无误后安全移除源卡文件
                fs::remove_file(src).with_context(|| {
                    format!("Failed to remove source file after move: {:?}", src)
                })?;
            }
        }

        Ok(())
    }

    /// 根据模板生成新文件名
    /// 支持占位符：{original}, {date}, {time}, {seq}, {seq3}, {seq4}, {camera}
    pub fn generate_target_base_name(
        item: &PhotoGroup,
        seq_num: usize,
        pattern: Option<&str>,
    ) -> String {
        let pattern = match pattern {
            Some(p) if !p.trim().is_empty() => p.trim(),
            _ => return item.base_name.clone(),
        };

        let date_str = item
            .capture_time
            .as_deref()
            .and_then(|t| t.split_whitespace().next())
            .map(|d| d.replace([':', '-'], ""))
            .unwrap_or_else(|| chrono::Local::now().format("%Y%m%d").to_string());

        let time_str = item
            .capture_time
            .as_deref()
            .and_then(|t| t.split_whitespace().nth(1))
            .map(|t| t.replace(':', ""))
            .unwrap_or_else(|| chrono::Local::now().format("%H%M%S").to_string());

        let camera_str = item
            .exif
            .as_ref()
            .and_then(|e| e.camera_model.as_deref())
            .unwrap_or("Camera")
            .replace(' ', "_");

        pattern
            .replace("{original}", &item.base_name)
            .replace("{date}", &date_str)
            .replace("{time}", &time_str)
            .replace("{camera}", &camera_str)
            .replace("{seq}", &seq_num.to_string())
            .replace("{seq3}", &format!("{:03}", seq_num))
            .replace("{seq4}", &format!("{:04}", seq_num))
            .replace("{seq5}", &format!("{:05}", seq_num))
    }

    /// 生成符合 Adobe Lightroom Classic 标准的 XMP 元数据侧车内容
    pub fn generate_xmp_content(rating: u8, color_label: &str, flag: i8) -> String {
        let lrc_label = match color_label.to_lowercase().as_str() {
            "red" => "Red",
            "yellow" => "Yellow",
            "green" => "Green",
            "blue" => "Blue",
            "purple" => "Purple",
            _ => "",
        };

        // Lightroom / Photoshop 旗标映射：Pick 对应 photoshop:Urgency="1"
        let urgency_tag = if flag == 1 {
            "photoshop:Urgency=\"1\""
        } else {
            ""
        };

        let label_tag = if !lrc_label.is_empty() {
            format!("xmp:Label=\"{}\"", lrc_label)
        } else {
            String::new()
        };

        format!(
            r#"<?xpacket begin="" id="W5M0MpCehiHzreSzNTczkc9d"?>
<x:xmpmeta xmlns:x="adobe:ns:meta/" x:xmptk="PhotoSelect Desktop 1.0">
 <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
  <rdf:Description rdf:about=""
    xmlns:xmp="http://ns.adobe.com/xap/1.0/"
    xmlns:photoshop="http://ns.adobe.com/photoshop/1.0/"
    xmp:Rating="{rating}"
    {label_tag}
    {urgency_tag}>
  </rdf:Description>
 </rdf:RDF>
</x:xmpmeta>
<?xpacket end="w"?>
"#
        )
    }
}
