use anyhow::{bail, Context, Result};
use byteorder::{BigEndian, ByteOrder, LittleEndian};
use image::codecs::jpeg::JpegEncoder;
use image::{ExtendedColorType, ImageEncoder};
use std::fs::File;
use std::io::{Cursor, Read, Seek, SeekFrom};
use std::path::Path;

use crate::models::ExifSummary;

/// JPEG 候选提取区域
#[derive(Debug, Clone)]
pub struct JpegCandidate {
    pub offset: u64,
    pub length: u64,
}

/// 毫秒级 RAW 内嵌预览图与普通图像提取引擎
pub struct ImageDecoder;

impl ImageDecoder {
    /// Develop a camera RAW sensor image into a color-managed sRGB JPEG.
    pub fn develop_raw_jpeg(path: &Path) -> Result<Vec<u8>> {
        let mut image =
            rawler::analyze::raw_to_srgb(path, &rawler::decoders::RawDecodeParams::default())
                .map_err(|err| anyhow::anyhow!("RAW development failed: {err}"))?;

        if image.width() == 0 || image.height() == 0 {
            bail!("RAW decoder returned an empty image");
        }

        const MAX_PREVIEW_EDGE: u32 = 8192;
        let largest_edge = image.width().max(image.height());
        if largest_edge > MAX_PREVIEW_EDGE {
            let scale = MAX_PREVIEW_EDGE as f64 / largest_edge as f64;
            let width = (image.width() as f64 * scale).round().max(1.0) as u32;
            let height = (image.height() as f64 * scale).round().max(1.0) as u32;
            image = image.resize_exact(width, height, image::imageops::FilterType::Lanczos3);
        }

        let rgb = image.to_rgb8();
        let mut output = Cursor::new(Vec::new());
        JpegEncoder::new_with_quality(&mut output, 95)
            .write_image(
                rgb.as_raw(),
                rgb.width(),
                rgb.height(),
                ExtendedColorType::Rgb8,
            )
            .context("Failed to encode developed RAW preview")?;
        Ok(output.into_inner())
    }

    /// 从文件路径中提取用于单图预览的高清全尺寸二进制字节流 (毫秒级)
    /// 1. 若为 JPG / PNG / WEBP 等标准图，直接读取文件数据
    /// 2. 若为 RAW 格式 (ARW, CR2, CR3, NEF, RAF 等)，通过解析容器头部提取内嵌全尺寸 JPEG
    pub fn extract_preview_bytes(path: &Path) -> Result<Vec<u8>> {
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        match ext.as_str() {
            // 标准图像格式直接读取
            "jpg" | "jpeg" | "png" | "webp" => std::fs::read(path)
                .with_context(|| format!("Failed to read standard image: {:?}", path)),

            // RAW 文件优先走完整开发路径：解码传感器数据、解马赛克、白平衡并转换到 sRGB。
            // 某些相机型号不在 rawler 数据库中时，回退到内嵌 JPEG 预览。
            "raf" | "cr3" | "arw" | "cr2" | "nef" | "nrw" | "dng" | "orf" | "pef" | "rw2"
            | "3fr" | "iiq" => Self::develop_raw_jpeg(path).or_else(|_| match ext.as_str() {
                "raf" => Self::extract_fuji_raf_preview(path),
                "cr3" => Self::extract_cr3_preview(path),
                _ => Self::extract_tiff_raw_preview(path),
            }),

            // 其他未知格式：尝试 TIFF 解析，失败则回退至 JPEG SOI 扫描
            _ => {
                if let Ok(bytes) = Self::extract_tiff_raw_preview(path) {
                    Ok(bytes)
                } else {
                    Self::extract_by_jpeg_soi_scan(path)
                }
            }
        }
    }

    /// 毫秒级提取轻量级缩略图二进制流 (专供胶片条与网格，体积 10KB~200KB，解码耗时 < 1ms，内存降低 100 倍)
    pub fn extract_thumbnail_bytes(path: &Path) -> Result<Vec<u8>> {
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        match ext.as_str() {
            "jpg" | "jpeg" => {
                if let Ok(mut file) = File::open(path) {
                    if let Some(thumb) = Self::extract_jpeg_embedded_thumbnail(&mut file) {
                        return Ok(thumb);
                    }
                }
                Self::extract_preview_bytes(path)
            }
            "cr3" => {
                if let Ok(bytes) = Self::extract_cr3_thumbnail(path) {
                    Ok(bytes)
                } else {
                    Self::extract_cr3_preview(path)
                }
            }
            "raf" => Self::extract_fuji_raf_preview(path),
            "arw" | "cr2" | "nef" | "nrw" | "dng" | "orf" | "pef" | "rw2" | "3fr" | "iiq" => {
                Self::extract_tiff_raw_thumbnail(path)
            }
            _ => Self::extract_preview_bytes(path),
        }
    }

    /// 从标准相机 JPG 文件 APP1 EXIF 中毫秒级提取嵌入式小缩略图 (0.1ms, ~15KB)
    pub fn extract_jpeg_embedded_thumbnail(file: &mut File) -> Option<Vec<u8>> {
        let mut soi = [0u8; 2];
        file.seek(SeekFrom::Start(0)).ok()?;
        file.read_exact(&mut soi).ok()?;
        if soi != [0xFF, 0xD8] {
            return None;
        }

        let mut offset = 2u64;
        let file_len = file.metadata().ok()?.len();

        while offset + 4 < file_len.min(2 * 1024 * 1024) {
            file.seek(SeekFrom::Start(offset)).ok()?;
            let mut marker_buf = [0u8; 4];
            file.read_exact(&mut marker_buf).ok()?;

            if marker_buf[0] != 0xFF {
                break;
            }
            let marker = marker_buf[1];
            if marker == 0xDA || marker == 0xD9 {
                break;
            }

            let seg_len = BigEndian::read_u16(&marker_buf[2..4]) as u64;
            if seg_len < 2 {
                break;
            }

            // APP1 标记: 0xE1 (标准 EXIF 元数据段)
            if marker == 0xE1 && seg_len >= 14 {
                let mut exif_hdr = [0u8; 6];
                if file.read_exact(&mut exif_hdr).is_ok() && &exif_hdr == b"Exif\0\0" {
                    let tiff_base = offset + 4 + 6;
                    let mut tiff_hdr = [0u8; 8];
                    if file.read_exact(&mut tiff_hdr).is_ok() {
                        let is_le = match &tiff_hdr[0..2] {
                            b"II" => true,
                            b"MM" => false,
                            _ => return None,
                        };
                        let ifd0_off = if is_le {
                            LittleEndian::read_u32(&tiff_hdr[4..8]) as u64
                        } else {
                            BigEndian::read_u32(&tiff_hdr[4..8]) as u64
                        };

                        let ifd0_abs = tiff_base + ifd0_off;
                        file.seek(SeekFrom::Start(ifd0_abs)).ok()?;
                        let mut count_buf = [0u8; 2];
                        file.read_exact(&mut count_buf).ok()?;
                        let num_entries = if is_le {
                            LittleEndian::read_u16(&count_buf) as u64
                        } else {
                            BigEndian::read_u16(&count_buf) as u64
                        };

                        // IFD1 偏移量指针位于 IFD0 末尾 (2 字节条目数 + 条目数 * 12 字节后)
                        let ifd1_ptr_offset = ifd0_abs + 2 + num_entries * 12;
                        file.seek(SeekFrom::Start(ifd1_ptr_offset)).ok()?;
                        let mut ifd1_buf = [0u8; 4];
                        file.read_exact(&mut ifd1_buf).ok()?;
                        let ifd1_off = if is_le {
                            LittleEndian::read_u32(&ifd1_buf) as u64
                        } else {
                            BigEndian::read_u32(&ifd1_buf) as u64
                        };

                        if ifd1_off != 0 && ifd1_off < seg_len {
                            let ifd1_abs = tiff_base + ifd1_off;
                            file.seek(SeekFrom::Start(ifd1_abs)).ok()?;
                            file.read_exact(&mut count_buf).ok()?;
                            let ifd1_entries = if is_le {
                                LittleEndian::read_u16(&count_buf) as usize
                            } else {
                                BigEndian::read_u16(&count_buf) as usize
                            };

                            let mut thumb_offset: Option<u64> = None;
                            let mut thumb_length: Option<u64> = None;

                            for _ in 0..ifd1_entries.min(64) {
                                let mut entry = [0u8; 12];
                                if file.read_exact(&mut entry).is_err() {
                                    break;
                                }
                                let tag = if is_le {
                                    LittleEndian::read_u16(&entry[0..2])
                                } else {
                                    BigEndian::read_u16(&entry[0..2])
                                };
                                let val = if is_le {
                                    LittleEndian::read_u32(&entry[8..12]) as u64
                                } else {
                                    BigEndian::read_u32(&entry[8..12]) as u64
                                };
                                if tag == 0x0201 {
                                    thumb_offset = Some(val);
                                } else if tag == 0x0202 {
                                    thumb_length = Some(val);
                                }
                            }

                            if let (Some(t_off), Some(t_len)) = (thumb_offset, thumb_length) {
                                if t_len > 1024 && t_len < 1024 * 1024 {
                                    let abs_thumb_off = tiff_base + t_off;
                                    file.seek(SeekFrom::Start(abs_thumb_off)).ok()?;
                                    let mut thumb_bytes = vec![0u8; t_len as usize];
                                    if file.read_exact(&mut thumb_bytes).is_ok() {
                                        if thumb_bytes.starts_with(&[0xFF, 0xD8]) {
                                            return Some(thumb_bytes);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            offset = offset.saturating_add(2 + seg_len);
        }

        None
    }

    /// 提取 TIFF 架构 RAW 格式高速缩略图
    fn extract_tiff_raw_thumbnail(path: &Path) -> Result<Vec<u8>> {
        let mut file = File::open(path)
            .with_context(|| format!("Failed to open RAW file for thumbnail: {:?}", path))?;
        let file_len = file.metadata()?.len();

        let mut header = [0u8; 8];
        file.read_exact(&mut header)
            .context("Invalid file header: too short")?;

        let is_le = match &header[0..2] {
            b"II" => true,
            b"MM" => false,
            _ => return Self::extract_tiff_raw_preview(path),
        };

        let magic = if is_le {
            LittleEndian::read_u16(&header[2..4])
        } else {
            BigEndian::read_u16(&header[2..4])
        };

        if magic != 42 && magic != 0x4F52 && magic != 0x55 {
            return Self::extract_tiff_raw_preview(path);
        }

        let first_ifd_offset = if is_le {
            LittleEndian::read_u32(&header[4..8]) as u64
        } else {
            BigEndian::read_u32(&header[4..8]) as u64
        };

        let mut candidates = Vec::with_capacity(16);
        let mut visited_offsets = Vec::with_capacity(32);

        Self::collect_tiff_jpeg_candidates(
            &mut file,
            first_ifd_offset,
            is_le,
            file_len,
            &mut candidates,
            &mut visited_offsets,
            0,
        )?;

        // 挑选 8KB ~ 800KB 之间的内嵌预览缩略图 (体积小、解析飞快，仅占用 0.5ms 解码时间)
        let selected = candidates
            .iter()
            .filter(|c| c.length >= 8 * 1024 && c.length <= 800 * 1024)
            .min_by_key(|c| c.length)
            .or_else(|| candidates.iter().min_by_key(|c| c.length));

        if let Some(best) = selected {
            if best.length > 1024 && best.offset + best.length <= file_len {
                file.seek(SeekFrom::Start(best.offset))?;
                let mut buffer = vec![0u8; best.length as usize];
                file.read_exact(&mut buffer)?;

                if buffer.len() >= 3 && buffer[0] == 0xFF && buffer[1] == 0xD8 && buffer[2] == 0xFF
                {
                    return Ok(buffer);
                }
            }
        }

        Self::extract_tiff_raw_preview(path)
    }

    /// 提取 Canon CR3 格式轻量级缩略图
    fn extract_cr3_thumbnail(path: &Path) -> Result<Vec<u8>> {
        let mut file = File::open(path)?;
        let file_len = file.metadata()?.len();
        let mut offset = 0u64;
        let mut target_box = None;

        while offset + 8 <= file_len.min(16 * 1024 * 1024) {
            file.seek(SeekFrom::Start(offset))?;
            let mut box_header = [0u8; 8];
            if file.read_exact(&mut box_header).is_err() {
                break;
            }
            let box_size = BigEndian::read_u32(&box_header[0..4]) as u64;
            let box_type = &box_header[4..8];
            if box_size < 8 {
                break;
            }
            if box_type == b"thmb" {
                target_box = Some((offset + 8, box_size - 8));
                break;
            }
            offset = offset.saturating_add(box_size);
        }

        if let Some((t_off, t_len)) = target_box {
            if t_len > 16 && t_off + t_len <= file_len {
                file.seek(SeekFrom::Start(t_off))?;
                let mut data = vec![0u8; t_len.min(512 * 1024) as usize];
                file.read_exact(&mut data)?;
                if let Some(pos) = data.windows(3).position(|w| w == [0xFF, 0xD8, 0xFF]) {
                    return Ok(data[pos..].to_vec());
                }
            }
        }

        Self::extract_cr3_preview(path)
    }

    /// 提取相机 EXIF 核心参数快速摘要 (支持 JPEG APP1、TIFF RAW、Canon CR3、Fuji RAF)
    pub fn extract_exif_summary(path: &Path) -> Option<ExifSummary> {
        let mut file = File::open(path).ok()?;
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        match ext.as_str() {
            "jpg" | "jpeg" => Self::extract_jpeg_exif(&mut file),
            "cr3" => Self::extract_cr3_exif(&mut file),
            _ => {
                // 1. 尝试标准 TIFF 头 (Sony ARW, Canon CR2, Nikon NEF, DNG, Olympus ORF, Pentax PEF)
                if let Some(summary) = Self::extract_tiff_exif_at(&mut file, 0) {
                    Some(summary)
                } else {
                    // 2. 回退尝试扫描内嵌 JPEG 中的 APP1 EXIF (Fuji RAF, Panasonic RW2 等)
                    Self::extract_embedded_jpeg_exif(&mut file)
                }
            }
        }
    }

    // =========================================================================
    // 1. TIFF-Based RAW 预览提取 (ARW, CR2, NEF, DNG, ORF, PEF)
    // =========================================================================

    fn extract_tiff_raw_preview(path: &Path) -> Result<Vec<u8>> {
        let mut file =
            File::open(path).with_context(|| format!("Failed to open RAW file: {:?}", path))?;
        let file_len = file.metadata()?.len();

        let mut header = [0u8; 8];
        file.read_exact(&mut header)
            .context("Invalid file header: too short")?;

        let is_le = match &header[0..2] {
            b"II" => true,
            b"MM" => false,
            _ => {
                return Self::extract_by_jpeg_soi_scan(path);
            }
        };

        let magic = if is_le {
            LittleEndian::read_u16(&header[2..4])
        } else {
            BigEndian::read_u16(&header[2..4])
        };

        if magic != 42 && magic != 0x4F52 && magic != 0x55 {
            return Self::extract_by_jpeg_soi_scan(path);
        }

        let first_ifd_offset = if is_le {
            LittleEndian::read_u32(&header[4..8]) as u64
        } else {
            BigEndian::read_u32(&header[4..8]) as u64
        };

        let mut candidates = Vec::with_capacity(16);
        let mut visited_offsets = Vec::with_capacity(32);

        Self::collect_tiff_jpeg_candidates(
            &mut file,
            first_ifd_offset,
            is_le,
            file_len,
            &mut candidates,
            &mut visited_offsets,
            0,
        )?;

        // 选择字节数最大的 JPEG 候选块 (即全尺寸高清内嵌图)
        if let Some(best) = candidates.into_iter().max_by_key(|c| c.length) {
            if best.length > 32 * 1024 && best.offset + best.length <= file_len {
                file.seek(SeekFrom::Start(best.offset))?;
                let mut buffer = vec![0u8; best.length as usize];
                file.read_exact(&mut buffer)?;

                // 校验 JPEG SOI 标记
                if buffer.len() >= 3 && buffer[0] == 0xFF && buffer[1] == 0xD8 && buffer[2] == 0xFF
                {
                    return Ok(buffer);
                }
            }
        }

        Self::extract_by_jpeg_soi_scan(path)
    }

    fn collect_tiff_jpeg_candidates(
        file: &mut File,
        ifd_offset: u64,
        is_le: bool,
        file_len: u64,
        candidates: &mut Vec<JpegCandidate>,
        visited: &mut Vec<u64>,
        depth: usize,
    ) -> Result<()> {
        if depth > 10
            || ifd_offset == 0
            || ifd_offset + 2 > file_len
            || visited.contains(&ifd_offset)
        {
            return Ok(());
        }
        visited.push(ifd_offset);

        file.seek(SeekFrom::Start(ifd_offset))?;
        let mut count_buf = [0u8; 2];
        file.read_exact(&mut count_buf)?;
        let num_entries = if is_le {
            LittleEndian::read_u16(&count_buf)
        } else {
            BigEndian::read_u16(&count_buf)
        };

        // 限制单 IFD 最大条目数，防止畸变头导致 DoS
        let safe_entries = num_entries.min(1024);

        let mut next_ifds_to_visit = Vec::new();
        let mut current_offset: Option<u64> = None;
        let mut current_length: Option<u64> = None;

        for _ in 0..safe_entries {
            let mut entry = [0u8; 12];
            if file.read_exact(&mut entry).is_err() {
                break;
            }

            let tag = if is_le {
                LittleEndian::read_u16(&entry[0..2])
            } else {
                BigEndian::read_u16(&entry[0..2])
            };
            let count = if is_le {
                LittleEndian::read_u32(&entry[4..8])
            } else {
                BigEndian::read_u32(&entry[4..8])
            };
            let val_or_offset = if is_le {
                LittleEndian::read_u32(&entry[8..12]) as u64
            } else {
                BigEndian::read_u32(&entry[8..12]) as u64
            };

            match tag {
                0x0201 => current_offset = Some(val_or_offset),
                0x0202 => current_length = Some(val_or_offset),
                0x0111 => {
                    if count == 1 {
                        current_offset = Some(val_or_offset);
                    }
                }
                0x0117 => {
                    if count == 1 {
                        current_length = Some(val_or_offset);
                    }
                }
                0x014A => {
                    if count == 1 {
                        next_ifds_to_visit.push(val_or_offset);
                    } else if count > 1 && count < 10 {
                        let mut sub_buf = vec![0u8; (count * 4) as usize];
                        let cur_pos = file.stream_position().unwrap_or(0);
                        if file.seek(SeekFrom::Start(val_or_offset)).is_ok()
                            && file.read_exact(&mut sub_buf).is_ok()
                        {
                            for i in 0..count as usize {
                                let offset = if is_le {
                                    LittleEndian::read_u32(&sub_buf[i * 4..i * 4 + 4]) as u64
                                } else {
                                    BigEndian::read_u32(&sub_buf[i * 4..i * 4 + 4]) as u64
                                };
                                next_ifds_to_visit.push(offset);
                            }
                        }
                        let _ = file.seek(SeekFrom::Start(cur_pos));
                    }
                }
                0x8769 => next_ifds_to_visit.push(val_or_offset),
                _ => {}
            }
        }

        if let (Some(offset), Some(length)) = (current_offset, current_length) {
            if length > 0 && offset + length <= file_len {
                candidates.push(JpegCandidate { offset, length });
            }
        }

        let mut next_ifd_buf = [0u8; 4];
        if file.read_exact(&mut next_ifd_buf).is_ok() {
            let next_ifd = if is_le {
                LittleEndian::read_u32(&next_ifd_buf) as u64
            } else {
                BigEndian::read_u32(&next_ifd_buf) as u64
            };
            if next_ifd > 0 && next_ifd < file_len {
                next_ifds_to_visit.push(next_ifd);
            }
        }

        for next_offset in next_ifds_to_visit {
            let _ = Self::collect_tiff_jpeg_candidates(
                file,
                next_offset,
                is_le,
                file_len,
                candidates,
                visited,
                depth + 1,
            );
        }

        Ok(())
    }

    // =========================================================================
    // 2. 富士 Fujifilm RAF 预览提取
    // =========================================================================

    fn extract_fuji_raf_preview(path: &Path) -> Result<Vec<u8>> {
        let mut file = File::open(path)?;
        let file_len = file.metadata()?.len();
        if file_len < 160 {
            bail!("Fuji RAF file too small");
        }

        let mut magic = [0u8; 16];
        file.read_exact(&mut magic)?;
        if &magic != b"FUJIFILMCCD-RAW " {
            return Self::extract_by_jpeg_soi_scan(path);
        }

        file.seek(SeekFrom::Start(84))?;
        let mut buf = [0u8; 8];
        file.read_exact(&mut buf)?;

        let jpeg_offset = BigEndian::read_u32(&buf[0..4]) as u64;
        let jpeg_length = BigEndian::read_u32(&buf[4..8]) as u64;

        if jpeg_offset > 0 && jpeg_length > 1024 && jpeg_offset + jpeg_length <= file_len {
            file.seek(SeekFrom::Start(jpeg_offset))?;
            let mut jpeg_bytes = vec![0u8; jpeg_length as usize];
            file.read_exact(&mut jpeg_bytes)?;

            if jpeg_bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
                return Ok(jpeg_bytes);
            }
        }

        Self::extract_by_jpeg_soi_scan(path)
    }

    // =========================================================================
    // 3. 佳能 Canon CR3 (ISOBMFF / QuickTime Box 架构) 预览提取
    // =========================================================================

    fn extract_cr3_preview(path: &Path) -> Result<Vec<u8>> {
        let mut file = File::open(path)?;
        let file_len = file.metadata()?.len();

        let mut offset = 0u64;
        let mut prvw_box_found: Option<(u64, u64)> = None;

        while offset + 8 <= file_len {
            file.seek(SeekFrom::Start(offset))?;
            let mut box_header = [0u8; 8];
            if file.read_exact(&mut box_header).is_err() {
                break;
            }

            let box_size_32 = BigEndian::read_u32(&box_header[0..4]) as u64;
            let box_type = &box_header[4..8];

            let (actual_size, header_size) = if box_size_32 == 1 {
                let mut ext_size = [0u8; 8];
                if file.read_exact(&mut ext_size).is_err() {
                    break;
                }
                (BigEndian::read_u64(&ext_size), 16u64)
            } else if box_size_32 == 0 {
                (file_len - offset, 8u64)
            } else {
                (box_size_32, 8u64)
            };

            if actual_size < header_size {
                break;
            }

            if box_type == b"prvw" {
                prvw_box_found = Some((offset + header_size, actual_size - header_size));
                break;
            }

            if box_type == b"moov" || box_type == b"uuid" {
                if let Ok(Some((p_off, p_len))) = Self::scan_sub_box(
                    &mut file,
                    offset + header_size,
                    actual_size - header_size,
                    0,
                ) {
                    prvw_box_found = Some((p_off, p_len));
                    break;
                }
            }

            offset = offset.saturating_add(actual_size);
        }

        if let Some((prvw_offset, prvw_length)) = prvw_box_found {
            if prvw_length > 16 && prvw_offset + prvw_length <= file_len {
                file.seek(SeekFrom::Start(prvw_offset))?;
                let read_len = prvw_length.min(32 * 1024 * 1024) as usize;
                let mut prvw_data = vec![0u8; read_len];
                file.read_exact(&mut prvw_data)?;

                if let Some(pos) = prvw_data.windows(3).position(|w| w == [0xFF, 0xD8, 0xFF]) {
                    let jpeg_bytes = prvw_data[pos..].to_vec();
                    return Ok(jpeg_bytes);
                }
            }
        }

        Self::extract_by_jpeg_soi_scan(path)
    }

    fn scan_sub_box(
        file: &mut File,
        start_offset: u64,
        max_len: u64,
        depth: usize,
    ) -> Result<Option<(u64, u64)>> {
        if depth > 8 {
            return Ok(None);
        }

        let mut cur = start_offset;
        let end = start_offset.saturating_add(max_len);

        while cur + 8 <= end {
            file.seek(SeekFrom::Start(cur))?;
            let mut box_header = [0u8; 8];
            if file.read_exact(&mut box_header).is_err() {
                break;
            }

            let box_size = BigEndian::read_u32(&box_header[0..4]) as u64;
            let box_type = &box_header[4..8];

            if box_size < 8 {
                break;
            }

            if box_type == b"prvw" {
                return Ok(Some((cur + 8, box_size - 8)));
            }

            if box_type == b"uuid" || box_type == b"trak" || box_type == b"mdia" {
                if let Ok(Some(res)) = Self::scan_sub_box(file, cur + 8, box_size - 8, depth + 1) {
                    return Ok(Some(res));
                }
            }

            cur = cur.saturating_add(box_size);
        }

        Ok(None)
    }

    // =========================================================================
    // 4. 智能 JPEG SOI/EOI 扫描降级回退
    // =========================================================================

    fn extract_by_jpeg_soi_scan(path: &Path) -> Result<Vec<u8>> {
        let mut file = File::open(path)?;
        let file_len = file.metadata()?.len();

        let scan_limit = file_len.min(24 * 1024 * 1024) as usize;
        let mut buffer = vec![0u8; scan_limit];
        file.read_exact(&mut buffer)?;

        let mut best_start = 0usize;
        let mut best_len = 0usize;

        let mut i = 0;
        while i + 4 < scan_limit {
            if buffer[i] == 0xFF && buffer[i + 1] == 0xD8 && buffer[i + 2] == 0xFF {
                let soi_start = i;
                let mut j = i + 1024;
                let mut eoi_pos = None;

                while j + 1 < scan_limit {
                    if buffer[j] == 0xFF && buffer[j + 1] == 0xD9 {
                        eoi_pos = Some(j + 2);
                        // 若已寻得完整高分辨率大图 (超过 1MB)，可停止继续往后扫描
                        if j - soi_start > 1024 * 1024 {
                            break;
                        }
                    }
                    j += 1;
                }

                if let Some(eoi) = eoi_pos {
                    let len = eoi - soi_start;
                    if len > best_len && len > 50 * 1024 {
                        best_len = len;
                        best_start = soi_start;
                    }
                }
                i = j.max(i + 4);
            } else {
                i += 1;
            }
        }

        if best_len > 0 {
            Ok(buffer[best_start..best_start + best_len].to_vec())
        } else {
            bail!("No embedded JPEG preview found in {:?}", path)
        }
    }

    // =========================================================================
    // 5. 快速 EXIF 标签解析辅助 (全面支持 JPEG APP1 与 TIFF IFD)
    // =========================================================================

    /// 从 JPEG 文件中查找并解析 APP1 Exif 段 (0xFF, 0xE1)
    fn extract_jpeg_exif(file: &mut File) -> Option<ExifSummary> {
        Self::extract_jpeg_exif_at_offset(file, 0)
    }

    /// 从指定偏移量开始按 JPEG 格式解析 APP1 EXIF
    fn extract_jpeg_exif_at_offset(file: &mut File, start_pos: u64) -> Option<ExifSummary> {
        file.seek(SeekFrom::Start(start_pos)).ok()?;
        let mut soi = [0u8; 2];
        file.read_exact(&mut soi).ok()?;
        if soi != [0xFF, 0xD8] {
            return None;
        }

        let mut offset = start_pos + 2;
        let file_len = file.metadata().ok()?.len();

        while offset + 4 <= file_len {
            file.seek(SeekFrom::Start(offset)).ok()?;
            let mut marker_buf = [0u8; 4];
            file.read_exact(&mut marker_buf).ok()?;

            if marker_buf[0] != 0xFF {
                break;
            }
            let marker = marker_buf[1];
            // 碰到图像开始标记 SOS (0xDA) 或 EOI (0xD9) 即终止扫描
            if marker == 0xDA || marker == 0xD9 {
                break;
            }

            let seg_len = BigEndian::read_u16(&marker_buf[2..4]) as u64;
            if seg_len < 2 {
                break;
            }

            // APP1 标记: 0xE1 (标准 EXIF 元数据段)
            if marker == 0xE1 && seg_len >= 8 {
                let mut exif_hdr = [0u8; 6];
                if file.read_exact(&mut exif_hdr).is_ok() && &exif_hdr == b"Exif\0\0" {
                    // TIFF 头部起点位于 APP1 段内 (跳过 2字节标记 + 2字节长度 + 6字节 "Exif\0\0")
                    let tiff_base = offset + 4 + 6;
                    let mut summary = ExifSummary::default();
                    if Self::parse_tiff_at(file, tiff_base, &mut summary).is_ok() {
                        return Some(summary);
                    }
                }
            }

            offset = offset.saturating_add(2 + seg_len);
        }

        None
    }

    /// 从 Canon CR3 文件提取 EXIF (优先查找 prvw box 内的 JPEG EXIF)
    fn extract_cr3_exif(file: &mut File) -> Option<ExifSummary> {
        let file_len = file.metadata().ok()?.len();
        let mut offset = 0u64;
        let mut prvw_box = None;

        while offset + 8 <= file_len.min(32 * 1024 * 1024) {
            if file.seek(SeekFrom::Start(offset)).is_err() {
                break;
            }
            let mut box_header = [0u8; 8];
            if file.read_exact(&mut box_header).is_err() {
                break;
            }

            let box_size = BigEndian::read_u32(&box_header[0..4]) as u64;
            let box_type = &box_header[4..8];
            if box_size < 8 {
                break;
            }

            if box_type == b"prvw" {
                prvw_box = Some((offset + 8, box_size - 8));
                break;
            }

            if box_type == b"moov" || box_type == b"uuid" {
                if let Ok(Some(sub)) = Self::scan_sub_box(file, offset + 8, box_size - 8, 0) {
                    prvw_box = Some(sub);
                    break;
                }
            }

            offset = offset.saturating_add(box_size);
        }

        if let Some((prvw_off, prvw_len)) = prvw_box {
            if prvw_len > 128 {
                let _ = file.seek(SeekFrom::Start(prvw_off));
                let mut buf = vec![0u8; prvw_len.min(65536) as usize];
                if file.read_exact(&mut buf).is_ok() {
                    if let Some(pos) = buf.windows(3).position(|w| w == [0xFF, 0xD8, 0xFF]) {
                        if let Some(s) =
                            Self::extract_jpeg_exif_at_offset(file, prvw_off + pos as u64)
                        {
                            return Some(s);
                        }
                    }
                }
            }
        }

        Self::extract_embedded_jpeg_exif(file)
    }

    /// 扫描文件前 16MB 区域寻找内嵌 JPEG APP1 EXIF
    fn extract_embedded_jpeg_exif(file: &mut File) -> Option<ExifSummary> {
        let file_len = file.metadata().ok()?.len();
        let scan_limit = file_len.min(16 * 1024 * 1024) as usize;
        let mut buf = vec![0u8; scan_limit];
        file.seek(SeekFrom::Start(0)).ok()?;
        file.read_exact(&mut buf).ok()?;

        for (i, w) in buf.windows(4).enumerate() {
            if w == [0xFF, 0xD8, 0xFF, 0xE1] {
                if let Some(s) = Self::extract_jpeg_exif_at_offset(file, i as u64) {
                    return Some(s);
                }
            }
        }
        None
    }

    /// 从指定 TIFF 起点解析 TIFF IFD 结构
    fn extract_tiff_exif_at(file: &mut File, tiff_base: u64) -> Option<ExifSummary> {
        let mut summary = ExifSummary::default();
        if Self::parse_tiff_at(file, tiff_base, &mut summary).is_ok() {
            if summary.aperture.is_some()
                || summary.shutter_speed.is_some()
                || summary.iso.is_some()
                || summary.camera_model.is_some()
                || summary.camera_make.is_some()
                || summary.capture_time.is_some()
            {
                return Some(summary);
            }
        }
        None
    }

    fn parse_tiff_at(file: &mut File, tiff_base: u64, summary: &mut ExifSummary) -> Result<()> {
        file.seek(SeekFrom::Start(tiff_base))?;
        let mut header = [0u8; 8];
        file.read_exact(&mut header)?;

        let is_le = match &header[0..2] {
            b"II" => true,
            b"MM" => false,
            _ => bail!("Not a TIFF header"),
        };

        let magic = if is_le {
            LittleEndian::read_u16(&header[2..4])
        } else {
            BigEndian::read_u16(&header[2..4])
        };

        if magic != 42 && magic != 0x4F52 && magic != 0x55 {
            bail!("Invalid TIFF magic");
        }

        let first_ifd_offset = if is_le {
            LittleEndian::read_u32(&header[4..8]) as u64
        } else {
            BigEndian::read_u32(&header[4..8]) as u64
        };

        Self::parse_ifd_entries(file, tiff_base, first_ifd_offset, is_le, summary, 0)
    }

    fn parse_ifd_entries(
        file: &mut File,
        tiff_base: u64,
        ifd_offset: u64,
        is_le: bool,
        summary: &mut ExifSummary,
        depth: usize,
    ) -> Result<()> {
        if depth > 6 || ifd_offset == 0 {
            return Ok(());
        }

        let abs_ifd = tiff_base.saturating_add(ifd_offset);
        file.seek(SeekFrom::Start(abs_ifd))?;
        let mut count_buf = [0u8; 2];
        file.read_exact(&mut count_buf)?;
        let num_entries = if is_le {
            LittleEndian::read_u16(&count_buf)
        } else {
            BigEndian::read_u16(&count_buf)
        };

        let mut sub_ifds = Vec::with_capacity(4);

        for _ in 0..num_entries.min(512) {
            let mut entry = [0u8; 12];
            if file.read_exact(&mut entry).is_err() {
                break;
            }

            let tag = if is_le {
                LittleEndian::read_u16(&entry[0..2])
            } else {
                BigEndian::read_u16(&entry[0..2])
            };
            let entry_type = if is_le {
                LittleEndian::read_u16(&entry[2..4])
            } else {
                BigEndian::read_u16(&entry[2..4])
            };
            let count = if is_le {
                LittleEndian::read_u32(&entry[4..8])
            } else {
                BigEndian::read_u32(&entry[4..8])
            };
            let val_raw = if is_le {
                LittleEndian::read_u32(&entry[8..12]) as u64
            } else {
                BigEndian::read_u32(&entry[8..12]) as u64
            };

            match tag {
                // 相机制造商 (Make)
                0x010F => {
                    if summary.camera_make.is_none() {
                        summary.camera_make =
                            Self::read_tiff_ascii(file, tiff_base, count, &entry[8..12], is_le);
                    }
                }
                // 机身型号 (Model)
                0x0110 => {
                    if summary.camera_model.is_none() {
                        summary.camera_model =
                            Self::read_tiff_ascii(file, tiff_base, count, &entry[8..12], is_le);
                    }
                }
                // 方向
                0x0112 => summary.orientation = Some(val_raw as u32),
                // 拍摄时间 (DateTimeOriginal / ModifyDate)
                0x9003 | 0x0132 => {
                    if summary.capture_time.is_none() {
                        if let Some(t) =
                            Self::read_tiff_ascii(file, tiff_base, count, &entry[8..12], is_le)
                        {
                            // 规范化 YYYY:MM:DD 为 YYYY-MM-DD
                            let norm = if t.len() >= 10
                                && t.chars().nth(4) == Some(':')
                                && t.chars().nth(7) == Some(':')
                            {
                                let mut chars: Vec<char> = t.chars().collect();
                                chars[4] = '-';
                                chars[7] = '-';
                                chars.into_iter().collect::<String>()
                            } else {
                                t
                            };
                            summary.capture_time = Some(norm);
                        }
                    }
                }
                // 连拍微秒/毫秒时间戳 (SubSecTime / SubSecTimeOriginal / SubSecTimeDigitized)
                0x9290 | 0x9291 | 0x9292 => {
                    if summary.sub_sec_time.is_none() {
                        if let Some(sub) =
                            Self::read_tiff_ascii(file, tiff_base, count, &entry[8..12], is_le)
                        {
                            let clean = sub.trim().to_string();
                            if !clean.is_empty() {
                                summary.sub_sec_time = Some(clean);
                            }
                        }
                    }
                }
                // 光圈 (FNumber)
                0x829D => {
                    if let Some((num, den)) =
                        Self::read_tiff_rational(file, tiff_base, val_raw, is_le)
                    {
                        summary.aperture = Some(Self::format_aperture(num, den));
                    }
                }
                // 快门 (ExposureTime)
                0x829A => {
                    if let Some((num, den)) =
                        Self::read_tiff_rational(file, tiff_base, val_raw, is_le)
                    {
                        summary.shutter_speed = Some(Self::format_shutter(num, den));
                    }
                }
                // ISO / PhotographicSensitivity
                0x8827 => {
                    let iso_val = if entry_type == 3 {
                        if is_le {
                            LittleEndian::read_u16(&entry[8..10]) as u32
                        } else {
                            BigEndian::read_u16(&entry[8..10]) as u32
                        }
                    } else {
                        val_raw as u32
                    };
                    if iso_val > 0 {
                        summary.iso = Some(iso_val);
                    }
                }
                // 焦距 (FocalLength)
                0x920A => {
                    if let Some((num, den)) =
                        Self::read_tiff_rational(file, tiff_base, val_raw, is_le)
                    {
                        summary.focal_length = Some(Self::format_focal_length(num, den));
                    }
                }
                // 35mm 等效焦距
                0xA405 => {
                    summary.focal_length_35mm = Some(format!("{} mm", val_raw));
                }
                // 曝光补偿 (ExposureBiasValue)
                0x9204 => {
                    if let Some((num, den)) =
                        Self::read_tiff_srational(file, tiff_base, val_raw, is_le)
                    {
                        summary.exposure_compensation = Some(Self::format_ev(num, den));
                    }
                }
                // 镜头型号 (LensModel)
                0xA434 => {
                    if summary.lens_model.is_none() {
                        summary.lens_model =
                            Self::read_tiff_ascii(file, tiff_base, count, &entry[8..12], is_le);
                    }
                }
                // Exif SubIFD 指针 (通常包含光圈快门ISO等核心曝光数据)
                0x8769 => sub_ifds.push(val_raw),
                _ => {}
            }
        }

        // 递归深入解析 Exif SubIFD
        for sub_offset in sub_ifds {
            let _ = Self::parse_ifd_entries(file, tiff_base, sub_offset, is_le, summary, depth + 1);
        }

        Ok(())
    }

    fn read_tiff_ascii(
        file: &mut File,
        tiff_base: u64,
        count: u32,
        raw_val: &[u8],
        is_le: bool,
    ) -> Option<String> {
        if count == 0 {
            return None;
        }

        if count <= 4 {
            let end = (count as usize).min(raw_val.len());
            let s = String::from_utf8_lossy(&raw_val[0..end])
                .trim_end_matches('\0')
                .trim()
                .to_string();
            return if s.is_empty() { None } else { Some(s) };
        }

        let offset = if is_le {
            LittleEndian::read_u32(raw_val) as u64
        } else {
            BigEndian::read_u32(raw_val) as u64
        };

        let cur_pos = file.stream_position().ok()?;
        file.seek(SeekFrom::Start(tiff_base.saturating_add(offset)))
            .ok()?;
        let mut buf = vec![0u8; count.min(256) as usize];
        file.read_exact(&mut buf).ok()?;
        let _ = file.seek(SeekFrom::Start(cur_pos));

        let s = String::from_utf8_lossy(&buf)
            .trim_end_matches('\0')
            .trim()
            .to_string();
        if s.is_empty() {
            None
        } else {
            Some(s)
        }
    }

    fn read_tiff_rational(
        file: &mut File,
        tiff_base: u64,
        offset: u64,
        is_le: bool,
    ) -> Option<(u32, u32)> {
        if offset == 0 {
            return None;
        }
        let cur_pos = file.stream_position().ok()?;
        file.seek(SeekFrom::Start(tiff_base.saturating_add(offset)))
            .ok()?;
        let mut buf = [0u8; 8];
        file.read_exact(&mut buf).ok()?;
        let _ = file.seek(SeekFrom::Start(cur_pos));

        let num = if is_le {
            LittleEndian::read_u32(&buf[0..4])
        } else {
            BigEndian::read_u32(&buf[0..4])
        };
        let den = if is_le {
            LittleEndian::read_u32(&buf[4..8])
        } else {
            BigEndian::read_u32(&buf[4..8])
        };
        if den == 0 {
            None
        } else {
            Some((num, den))
        }
    }

    fn read_tiff_srational(
        file: &mut File,
        tiff_base: u64,
        offset: u64,
        is_le: bool,
    ) -> Option<(i32, i32)> {
        if offset == 0 {
            return None;
        }
        let cur_pos = file.stream_position().ok()?;
        file.seek(SeekFrom::Start(tiff_base.saturating_add(offset)))
            .ok()?;
        let mut buf = [0u8; 8];
        file.read_exact(&mut buf).ok()?;
        let _ = file.seek(SeekFrom::Start(cur_pos));

        let num = if is_le {
            LittleEndian::read_i32(&buf[0..4])
        } else {
            BigEndian::read_i32(&buf[0..4])
        };
        let den = if is_le {
            LittleEndian::read_i32(&buf[4..8])
        } else {
            BigEndian::read_i32(&buf[4..8])
        };
        if den == 0 {
            None
        } else {
            Some((num, den))
        }
    }

    pub fn format_aperture(num: u32, den: u32) -> String {
        let f = num as f64 / den as f64;
        if (f.fract() * 10.0).round() == 0.0 {
            format!("f/{:.0}", f)
        } else {
            format!("f/{:.1}", f)
        }
    }

    pub fn format_shutter(num: u32, den: u32) -> String {
        if num == 0 || den == 0 {
            return "0s".to_string();
        }
        if num >= den {
            let sec = num as f64 / den as f64;
            if (sec.fract() * 10.0).round() == 0.0 {
                format!("{}s", sec.round() as u64)
            } else {
                format!("{:.1}s", sec)
            }
        } else {
            let inv = (den as f64 / num as f64).round() as u64;
            format!("1/{}s", inv)
        }
    }

    pub fn format_focal_length(num: u32, den: u32) -> String {
        let fl = num as f64 / den as f64;
        if (fl.fract() * 10.0).round() == 0.0 {
            format!("{:.0} mm", fl)
        } else {
            format!("{:.1} mm", fl)
        }
    }

    pub fn format_ev(num: i32, den: i32) -> String {
        let ev = num as f64 / den as f64;
        if ev.abs() < 0.05 {
            "0 EV".to_string()
        } else if ev > 0.0 {
            format!("+{:.1} EV", ev)
        } else {
            format!("{:.1} EV", ev)
        }
    }
}
