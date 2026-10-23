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

        // 生成 Adobe Lightroom Classic 兼容的 .xmp 侧车文件 (包含生态/通用关键字)
        if config.generate_xmp {
            let xmp_content = Self::generate_xmp_content(item.rating, &item.color_label, item.flag, &item.tags);

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

    /// 对 XML 文本进行特殊字符转义 (&, <, >, ", ')
    fn xml_escape(text: &str) -> String {
        text.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&apos;")
    }

    /// 生成符合 Adobe Lightroom Classic、Bridge、Capture One 标准的 XMP 元数据侧车内容
    ///
    /// 包含：
    /// - `xmp:Rating` (0-5星)
    /// - `xmp:Label` (色彩标签)
    /// - `photoshop:Urgency="1"` (Pick 留用旗标)
    /// - `dc:subject` (Dublin Core 平铺关键字列表，支持中英文鸟种、学名、特征)
    /// - `lr:hierarchicalSubject` (Lightroom 层级关键字，如 "动物|鸟类|红嘴相思鸟")
    pub fn generate_xmp_content(
        rating: u8,
        color_label: &str,
        flag: i8,
        keywords: &[String],
    ) -> String {
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

        // 整理平铺关键字与层级关键字
        let mut flat_keywords: Vec<String> = Vec::new();
        let mut hierarchical_keywords: Vec<String> = Vec::new();

        for kw in keywords {
            let trimmed = kw.trim();
            if trimmed.is_empty() {
                continue;
            }

            if trimmed.contains('|') {
                hierarchical_keywords.push(Self::xml_escape(trimmed));
                // 提取层级末尾叶子节点也放入平铺关键字
                if let Some(leaf) = trimmed.split('|').last() {
                    let clean_leaf = leaf.trim();
                    let escaped_leaf = Self::xml_escape(clean_leaf);
                    if !clean_leaf.is_empty() && !flat_keywords.contains(&escaped_leaf) {
                        flat_keywords.push(escaped_leaf);
                    }
                }
            } else {
                let escaped = Self::xml_escape(trimmed);
                if !flat_keywords.contains(&escaped) {
                    flat_keywords.push(escaped);
                }
            }
        }

        let dc_subject_xml = if !flat_keywords.is_empty() {
            let mut lis = String::new();
            for kw in &flat_keywords {
                lis.push_str(&format!("     <rdf:li>{}</rdf:li>\n", kw));
            }
            format!(
                "   <dc:subject>\n    <rdf:Bag>\n{}    </rdf:Bag>\n   </dc:subject>\n",
                lis
            )
        } else {
            String::new()
        };

        let lr_hierarchical_xml = if !hierarchical_keywords.is_empty() {
            let mut lis = String::new();
            for kw in &hierarchical_keywords {
                lis.push_str(&format!("     <rdf:li>{}</rdf:li>\n", kw));
            }
            format!(
                "   <lr:hierarchicalSubject>\n    <rdf:Bag>\n{}    </rdf:Bag>\n   </lr:hierarchicalSubject>\n",
                lis
            )
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
    xmlns:dc="http://purl.org/dc/elements/1.1/"
    xmlns:lr="http://ns.adobe.com/lightroom/1.0/"
    xmp:Rating="{rating}"
    {label_tag}
    {urgency_tag}>
{dc_subject_xml}{lr_hierarchical_xml}  </rdf:Description>
 </rdf:RDF>
</x:xmpmeta>
<?xpacket end="w"?>
"#
        )
    }

    /// 为指定单张照片在其原图同目录下生成/更新同名 .xmp 侧车文件 (直接回写)
    pub fn write_xmp_sidecar_for_photo(
        item: &PhotoGroup,
        extra_keywords: Option<&[String]>,
    ) -> Result<PathBuf> {
        let mut all_tags = item.tags.clone();
        if let Some(extra) = extra_keywords {
            for tag in extra {
                if !all_tags.contains(tag) {
                    all_tags.push(tag.clone());
                }
            }
        }

        let xmp_content = Self::generate_xmp_content(item.rating, &item.color_label, item.flag, &all_tags);

        // 首选与 RAW 同目录同名，无 RAW 则与 JPG 同名
        let target_dir = &item.folder_path;
        let xmp_path = target_dir.join(format!("{}.xmp", item.base_name));

        let mut file = File::create(&xmp_path)
            .with_context(|| format!("Failed to create XMP file: {:?}", xmp_path))?;
        file.write_all(xmp_content.as_bytes())?;
        file.sync_all()?;

        Ok(xmp_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xmp_content_generation() {
        let tags = vec![
            "红嘴相思鸟".to_string(),
            "Leiothrix lutea".to_string(),
            "动物|鸟类|红嘴相思鸟".to_string(),
        ];

        let xmp = Exporter::generate_xmp_content(5, "green", 1, &tags);

        assert!(xmp.contains(r#"xmp:Rating="5""#));
        assert!(xmp.contains(r#"photoshop:Urgency="1""#));
        assert!(xmp.contains(r#"xmp:Label="Green""#));
        // Flat keywords
        assert!(xmp.contains("<dc:subject>"));
        assert!(xmp.contains("<rdf:li>红嘴相思鸟</rdf:li>"));
        assert!(xmp.contains("<rdf:li>Leiothrix lutea</rdf:li>"));
        // Hierarchical keywords
        assert!(xmp.contains("<lr:hierarchicalSubject>"));
        assert!(xmp.contains("<rdf:li>动物|鸟类|红嘴相思鸟</rdf:li>"));
    }
}
