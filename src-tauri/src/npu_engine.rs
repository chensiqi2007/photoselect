//! Open-Core AI & Hardware Acceleration Interface Layer
//!
//! - When compiled with the closed-source commercial kernel (`src/proprietary/npu_kernel.rs`,
//!   automatically detected by `build.rs` via `#[cfg(has_proprietary_ai)]`), this module activates
//!   Intel® oneAPI Level Zero NPU acceleration, W8A8 INT8 quantized vision neural embeddings,
//!   and neural focus-sharpness Best Shot ranking.
//! - In the public open-source repository (where `src/proprietary/` is omitted), it compiles a
//!   clean community fallback stub using standard 64-bit dHash.

use image::DynamicImage;
use serde::{Deserialize, Serialize};

use crate::models::PhotoGroup;

#[cfg(has_proprietary_ai)]
#[path = "proprietary/npu_kernel.rs"]
pub(crate) mod kernel;

pub const EMBED_DIM: usize = 128;

/// 用户可选的算力驱动模式 (Compute Backend Driver Mode)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ComputeBackend {
    #[default]
    Auto,
    Npu,
    Gpu,
    CpuAi,
    CpuClassic,
}

impl ComputeBackend {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Npu => "npu",
            Self::Gpu => "gpu",
            Self::CpuAi => "cpu_ai",
            Self::CpuClassic => "cpu_classic",
        }
    }

    pub fn from_str_opt(s: Option<&str>) -> Self {
        match s.map(|v| v.trim().to_ascii_lowercase()).as_deref() {
            Some("npu") => Self::Npu,
            Some("gpu") => Self::Gpu,
            Some("cpu_ai") => Self::CpuAi,
            Some("cpu_classic") | Some("cpu") => Self::CpuClassic,
            _ => Self::Auto,
        }
    }
}

/// 单个硬件加速设备探测详情
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComputeDeviceInfo {
    pub backend: String,
    pub device_name: String,
    pub vendor: String,
    pub driver_api: String,
    pub available: bool,
    pub specs: String,
    pub recommended: bool,
}

/// 全局算力设备枚举与当前配置状态响应
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComputeCapabilities {
    pub selected_backend: String,
    pub active_backend: String,
    pub active_device_name: String,
    pub npu_available: bool,
    pub gpu_available: bool,
    pub level_zero_loaded: bool,
    pub directml_loaded: bool,
    pub devices: Vec<ComputeDeviceInfo>,
}

/// 单次聚合/聚类推理的性能遥测指标
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct InferenceTelemetry {
    pub requested_backend: String,
    pub executed_backend: String,
    pub device_name: String,
    pub precision_mode: String,
    pub photo_count: usize,
    pub total_elapsed_ms: u64,
    pub avg_ms_per_photo: f32,
}

/// 单张照片的特征提取结果结构
#[derive(Debug, Clone)]
pub struct PhotoNeuralFeatures {
    pub dhash: u64,
    pub embedding: [f32; EMBED_DIM],
    pub sharpness_score: f32,
    pub exposure_score: f32,
    pub quality_score: u8,
    pub scene_tag: &'static str,
}

pub struct NpuEngine;

impl NpuEngine {
    pub fn probe_capabilities(selected_pref: Option<&str>) -> ComputeCapabilities {
        #[cfg(has_proprietary_ai)]
        {
            kernel::probe_capabilities(selected_pref)
        }
        #[cfg(not(has_proprietary_ai))]
        {
            let pref = ComputeBackend::from_str_opt(selected_pref);
            ComputeCapabilities {
                selected_backend: pref.as_str().to_string(),
                active_backend: "cpu_classic".to_string(),
                active_device_name: "Community Open-Core (CPU dHash)".to_string(),
                npu_available: false,
                gpu_available: false,
                level_zero_loaded: false,
                directml_loaded: false,
                devices: vec![ComputeDeviceInfo {
                    backend: "cpu_classic".to_string(),
                    device_name: "CPU 64-bit dHash (Community Open-Core)".to_string(),
                    vendor: "CPU".to_string(),
                    driver_api: "Open-Core Fallback".to_string(),
                    available: true,
                    specs: "Proprietary NPU/AI Kernel not bundled in public repo".to_string(),
                    recommended: true,
                }],
            }
        }
    }

    pub fn extract_batch_features(
        photos: &[PhotoGroup],
        requested_backend: ComputeBackend,
    ) -> (Vec<Option<PhotoNeuralFeatures>>, InferenceTelemetry) {
        #[cfg(has_proprietary_ai)]
        {
            kernel::extract_batch_features(photos, requested_backend)
        }
        #[cfg(not(has_proprietary_ai))]
        {
            let telemetry = InferenceTelemetry {
                requested_backend: requested_backend.as_str().to_string(),
                executed_backend: "cpu_classic".to_string(),
                device_name: "Community Open-Core (CPU dHash)".to_string(),
                precision_mode: "64-bit Perceptual dHash".to_string(),
                photo_count: photos.len(),
                total_elapsed_ms: 0,
                avg_ms_per_photo: 0.0,
            };
            (vec![None; photos.len()], telemetry)
        }
    }

    pub fn analyze_image(img: &DynamicImage, backend: ComputeBackend) -> PhotoNeuralFeatures {
        #[cfg(has_proprietary_ai)]
        {
            kernel::analyze_image(img, backend)
        }
        #[cfg(not(has_proprietary_ai))]
        {
            let _ = backend;
            let dhash = crate::similarity::SimilarityDetector::compute_dhash_from_image(img);
            PhotoNeuralFeatures {
                dhash,
                embedding: [0.0; EMBED_DIM],
                sharpness_score: 50.0,
                exposure_score: 50.0,
                quality_score: 50,
                scene_tag: "基础模式",
            }
        }
    }

    #[inline]
    pub fn cosine_similarity(a: &[f32; EMBED_DIM], b: &[f32; EMBED_DIM]) -> f32 {
        #[cfg(has_proprietary_ai)]
        {
            kernel::cosine_similarity(a, b)
        }
        #[cfg(not(has_proprietary_ai))]
        {
            let _ = (a, b);
            0.0
        }
    }
}

#[cfg(all(test, has_proprietary_ai))]
mod tests {
    use super::*;
    use image::{Rgb, RgbImage};

    #[test]
    fn test_npu_hardware_probe_and_fallback() {
        let caps = NpuEngine::probe_capabilities(Some("auto"));
        assert!(!caps.devices.is_empty());
        assert!(!caps.active_device_name.is_empty());
    }

    #[test]
    fn test_int8_quantized_neural_embedding_and_similarity() {
        let mut img1 = RgbImage::new(128, 128);
        let mut img2 = RgbImage::new(128, 128);
        let mut img_diff = RgbImage::new(128, 128);

        for y in 0..128 {
            for x in 0..128 {
                let in_subj1 = (36..92).contains(&x) && (30..100).contains(&y);
                img1.put_pixel(
                    x,
                    y,
                    if in_subj1 {
                        Rgb([220, 140, 110])
                    } else {
                        Rgb([25, 30, 35])
                    },
                );

                let in_subj2 = (42..98).contains(&x) && (32..102).contains(&y);
                img2.put_pixel(
                    x,
                    y,
                    if in_subj2 {
                        Rgb([218, 138, 112])
                    } else {
                        Rgb([26, 31, 36])
                    },
                );

                img_diff.put_pixel(
                    x,
                    y,
                    if y < 64 {
                        Rgb([40, 130, 240])
                    } else {
                        Rgb([240, 245, 250])
                    },
                );
            }
        }

        let feat1 = NpuEngine::analyze_image(&DynamicImage::ImageRgb8(img1), ComputeBackend::Npu);
        let feat2 = NpuEngine::analyze_image(&DynamicImage::ImageRgb8(img2), ComputeBackend::Npu);
        let feat_diff =
            NpuEngine::analyze_image(&DynamicImage::ImageRgb8(img_diff), ComputeBackend::Npu);

        let sim_same = NpuEngine::cosine_similarity(&feat1.embedding, &feat2.embedding);
        let sim_diff = NpuEngine::cosine_similarity(&feat1.embedding, &feat_diff.embedding);

        assert!(sim_same > 0.90);
        assert!(sim_same > sim_diff + 0.25);
    }

    #[test]
    fn test_sharp_subject_beats_noisy_blurry_image() {
        let mut sharp_img = RgbImage::new(128, 128);
        let mut blurry_img = RgbImage::new(128, 128);

        for y in 0..128 {
            for x in 0..128 {
                let is_center = (32..96).contains(&x) && (32..96).contains(&y);
                let val = if is_center {
                    if ((x / 2) + (y / 2)) % 2 == 0 {
                        230
                    } else {
                        30
                    }
                } else {
                    100
                };
                sharp_img.put_pixel(x, y, Rgb([val, val, val]));
                blurry_img.put_pixel(x, y, Rgb([125, 125, 125]));
            }
        }

        let f_sharp =
            NpuEngine::analyze_image(&DynamicImage::ImageRgb8(sharp_img), ComputeBackend::Npu);
        let f_blur =
            NpuEngine::analyze_image(&DynamicImage::ImageRgb8(blurry_img), ComputeBackend::Npu);

        assert!(f_sharp.sharpness_score > f_blur.sharpness_score + 30.0);
        assert!(f_sharp.quality_score > f_blur.quality_score);
    }
}
