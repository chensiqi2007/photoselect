import { FaceLandmarker, FilesetResolver, type NormalizedLandmark } from "@mediapipe/tasks-vision";
import { invoke } from "@tauri-apps/api/core";
import type { DetectedEye, DetectedFace, EyeDetectionResult } from "../types/eye";
import type { InferenceTelemetry } from "../types/photo";

interface EyePatchInput {
  id: string;
  cropDataUrl: string;
  isOpen: boolean;
  fallbackSharpness: number;
}

interface EyeNeuralScore {
  id: string;
  sharpness: number;
  sharpnessLabel: "sharp" | "good" | "soft";
  diagnosis: string;
}

interface EyeFocusBatchResponse {
  scores: EyeNeuralScore[];
  telemetry: InferenceTelemetry;
}

const cache = new Map<string, EyeDetectionResult>();
let enginePromise: Promise<FaceLandmarker> | null = null;

async function getEngine(): Promise<FaceLandmarker> {
  if (!enginePromise) {
    enginePromise = (async () => {
      const files = await FilesetResolver.forVisionTasks("/vision");
      const options = {
        runningMode: "IMAGE" as const, numFaces: 6,
        minFaceDetectionConfidence: 0.35, minFacePresenceConfidence: 0.35,
        outputFaceBlendshapes: true,
      };
      try {
        return await FaceLandmarker.createFromOptions(files, {
          ...options, baseOptions: { modelAssetPath: "/models/face_landmarker.task", delegate: "GPU" },
        });
      } catch {
        return FaceLandmarker.createFromOptions(files, {
          ...options, baseOptions: { modelAssetPath: "/models/face_landmarker.task", delegate: "CPU" },
        });
      }
    })().catch((error) => { enginePromise = null; throw error; });
  }
  return enginePromise;
}

export function clearEyeDetectionCache() { cache.clear(); }

function crop(img: HTMLImageElement, x: number, y: number, size: number) {
  const side = Math.max(1, Math.min(Math.round(size), img.naturalWidth, img.naturalHeight));
  const box = {
    x: Math.max(0, Math.min(img.naturalWidth - side, Math.round(x - side / 2))),
    y: Math.max(0, Math.min(img.naturalHeight - side, Math.round(y - side / 2))),
    width: side, height: side,
  };
  const canvas = document.createElement("canvas");
  // Score native pixels without enlarging small eyes and inventing sharpness.
  canvas.width = canvas.height = side;
  const ctx = canvas.getContext("2d", { willReadFrequently: true });
  if (!ctx) throw new Error("Canvas image analysis is unavailable");
  ctx.drawImage(img, box.x, box.y, side, side, 0, 0, side, side);
  const { data } = ctx.getImageData(0, 0, side, side);
  const gray = (i: number) => data[i * 4] * 0.299 + data[i * 4 + 1] * 0.587 + data[i * 4 + 2] * 0.114;
  let sum = 0, square = 0, count = 0;
  for (let row = 1; row < side - 1; row++) {
    for (let col = 1; col < side - 1; col++) {
      const i = row * side + col;
      const lap = gray(i - 1) + gray(i + 1) + gray(i - side) + gray(i + side) - 4 * gray(i);
      sum += lap; square += lap * lap; count++;
    }
  }
  const score = count ? Math.round(Math.min(100, Math.sqrt(Math.max(0, square / count - (sum / count) ** 2)) * 2)) : 0;
  return { box, score, url: canvas.toDataURL("image/jpeg", 0.9) };
}

function eyeFromLandmarks(img: HTMLImageElement, points: NormalizedLandmark[], indices: number[], type: "left" | "right", blink?: number): DetectedEye | null {
  const p = indices.map((i) => points[i]);
  if (p.some((v) => !v || !Number.isFinite(v.x) || !Number.isFinite(v.y) || v.x < 0 || v.x > 1 || v.y < 0 || v.y > 1)) return null;
  const distance = (a: NormalizedLandmark, b: NormalizedLandmark) => Math.hypot((a.x - b.x) * img.naturalWidth, (a.y - b.y) * img.naturalHeight);
  const width = distance(p[0], p[3]);
  if (width < 8) return null;
  const ratio = (distance(p[1], p[5]) + distance(p[2], p[4])) / (2 * width);
  const x = (p[0].x + p[3].x) / 2, y = (p[0].y + p[3].y) / 2;
  const patch = crop(img, x * img.naturalWidth, y * img.naturalHeight, Math.min(192, Math.max(32, width * 1.5)));
  return {
    type, x, y, box: patch.box, sharpness: patch.score,
    sharpnessLabel: patch.score >= 75 ? "sharp" : patch.score >= 50 ? "good" : "soft",
    isOpen: blink === undefined ? ratio >= 0.18 : blink < 0.5,
    opennessRatio: blink === undefined ? Math.min(1, ratio / 0.3) : 1 - blink,
    cropDataUrl: patch.url,
  };
}

interface YunetFaceDetection {
  id: number;
  centerX: number;
  centerY: number;
  boxX: number;
  boxY: number;
  boxWidth: number;
  boxHeight: number;
  faceCropDataUrl: string;
  rightEyeX: number;
  rightEyeY: number;
  rightEyeCropDataUrl: string;
  rightEyeSharpness: number;
  rightEyeLabel: "sharp" | "good" | "soft";
  rightEyeDiagnosis: string;
  leftEyeX: number;
  leftEyeY: number;
  leftEyeCropDataUrl: string;
  leftEyeSharpness: number;
  leftEyeLabel: "sharp" | "good" | "soft";
  leftEyeDiagnosis: string;
  score: number;
}

export async function detectEyesInImage(
  img: HTMLImageElement,
  photoId: string,
  isCurrent: () => boolean = () => true,
  computeBackend: string = "auto",
  rawFilePath?: string | null
): Promise<EyeDetectionResult> {
  if (!isCurrent()) throw new DOMException("Superseded image", "AbortError");
  const key = `${computeBackend}:${photoId}:${img.src}`;
  const cached = cache.get(key);
  if (cached) { cache.delete(key); cache.set(key, cached); return cached; }
  const engine = await getEngine();
  if (!isCurrent()) throw new DOMException("Superseded image", "AbortError");
  const start = performance.now();
  const canvas = document.createElement("canvas");
  const scale = Math.min(1, 1920 / Math.max(img.naturalWidth, img.naturalHeight));
  canvas.width = Math.max(1, Math.round(img.naturalWidth * scale));
  canvas.height = Math.max(1, Math.round(img.naturalHeight * scale));
  const context = canvas.getContext("2d");
  if (!context) throw new Error("Canvas image analysis is unavailable");
  context.drawImage(img, 0, 0, canvas.width, canvas.height);
  const detected = engine.detect(canvas);
  const faces: DetectedFace[] = [];
  detected.faceLandmarks.forEach((points, i) => {
    const categories = detected.faceBlendshapes[i]?.categories ?? [];
    const blink = (name: string) => categories.find((c) => c.categoryName === name)?.score;
    const leftEye = eyeFromLandmarks(img, points, [362, 385, 387, 263, 373, 380], "left", blink("eyeBlinkLeft"));
    const rightEye = eyeFromLandmarks(img, points, [33, 160, 158, 133, 153, 144], "right", blink("eyeBlinkRight"));
    const eyes = [leftEye, rightEye].filter((v): v is DetectedEye => v !== null);
    if (!eyes.length) return;
    const x = Math.max(0, Math.min(...points.map((p) => p.x))) * img.naturalWidth;
    const y = Math.max(0, Math.min(...points.map((p) => p.y))) * img.naturalHeight;
    const width = Math.min(img.naturalWidth - x, Math.max(...points.map((p) => p.x)) * img.naturalWidth - x);
    const height = Math.min(img.naturalHeight - y, Math.max(...points.map((p) => p.y)) * img.naturalHeight - y);
    const thumb = document.createElement("canvas");
    thumb.width = thumb.height = 120;
    thumb.getContext("2d")?.drawImage(img, x, y, width, height, 0, 0, 120, 120);
    faces.push({
      id: i + 1, centerX: (x + width / 2) / img.naturalWidth, centerY: (y + height / 2) / img.naturalHeight,
      box: { x, y, width, height },
      faceCropDataUrl: thumb.toDataURL("image/jpeg", 0.85), leftEye, rightEye,
      primaryEye: eyes.reduce((a, b) => a.sharpness >= b.sharpness ? a : b),
      overallScore: Math.round(eyes.reduce((sum, e) => sum + e.sharpness, 0) / eyes.length),
      isBlinking: eyes.some((e) => !e.isOpen),
    });
  });

  // 若 MediaPipe 在中远景或戴帽人像上未检出，自动接管调用 Rust 端 MIT 许可 YuNet ONNX 模型
  if (faces.length === 0 && rawFilePath && isCurrent()) {
    try {
      const yunetFaces = await invoke<YunetFaceDetection[]>("detect_faces_yunet", {
        path: rawFilePath,
        computeBackend,
      });
      for (const yf of yunetFaces) {
        const leftEye: DetectedEye = {
          type: "left",
          x: yf.leftEyeX,
          y: yf.leftEyeY,
          box: { x: yf.leftEyeX * img.naturalWidth - 32, y: yf.leftEyeY * img.naturalHeight - 32, width: 64, height: 64 },
          sharpness: yf.leftEyeSharpness,
          sharpnessLabel: yf.leftEyeLabel,
          focusDiagnosis: yf.leftEyeDiagnosis,
          isOpen: true,
          opennessRatio: 1,
          cropDataUrl: yf.leftEyeCropDataUrl,
        };
        const rightEye: DetectedEye = {
          type: "right",
          x: yf.rightEyeX,
          y: yf.rightEyeY,
          box: { x: yf.rightEyeX * img.naturalWidth - 32, y: yf.rightEyeY * img.naturalHeight - 32, width: 64, height: 64 },
          sharpness: yf.rightEyeSharpness,
          sharpnessLabel: yf.rightEyeLabel,
          focusDiagnosis: yf.rightEyeDiagnosis,
          isOpen: true,
          opennessRatio: 1,
          cropDataUrl: yf.rightEyeCropDataUrl,
        };
        const primaryEye = leftEye.sharpness >= rightEye.sharpness ? leftEye : rightEye;
        faces.push({
          id: yf.id,
          centerX: yf.centerX,
          centerY: yf.centerY,
          box: { x: yf.boxX, y: yf.boxY, width: yf.boxWidth, height: yf.boxHeight },
          faceCropDataUrl: yf.faceCropDataUrl,
          leftEye,
          rightEye,
          primaryEye,
          overallScore: Math.round((leftEye.sharpness + rightEye.sharpness) / 2),
          isBlinking: false,
        });
      }
      if (faces.length > 0) {
        const result: EyeDetectionResult = {
          photoId,
          photoPath: img.src,
          faces,
          detectionTimeMs: Math.round(performance.now() - start),
          computeDevice: "YuNet (MIT) + EyeMicroFocusNet",
        };
        if (isCurrent()) {
          cache.set(key, result);
          while (cache.size > 24) cache.delete(cache.keys().next().value!);
        }
        return result;
      }
    } catch (err) {
      console.debug("YuNet 兜底检测跳过:", err);
    }
  }
  // 调用 Rust NPU / 神经大模型引擎对每只眼睛的 100% 原生像素裁切块进行深度合焦评估
  let computeDevice: string | undefined;
  let precisionMode: string | undefined;
  if (faces.length > 0 && isCurrent()) {
    const patches: EyePatchInput[] = [];
    faces.forEach((face) => {
      if (face.leftEye) {
        patches.push({
          id: `${face.id}_left`,
          cropDataUrl: face.leftEye.cropDataUrl,
          isOpen: face.leftEye.isOpen,
          fallbackSharpness: face.leftEye.sharpness,
        });
      }
      if (face.rightEye) {
        patches.push({
          id: `${face.id}_right`,
          cropDataUrl: face.rightEye.cropDataUrl,
          isOpen: face.rightEye.isOpen,
          fallbackSharpness: face.rightEye.sharpness,
        });
      }
    });

    if (patches.length > 0) {
      try {
        const res = await invoke<EyeFocusBatchResponse>("evaluate_eye_focus_neural", {
          patches,
          computeBackend,
        });
        const scoreMap = new Map(res.scores.map((s) => [s.id, s]));
        computeDevice = res.telemetry.deviceName;
        precisionMode = res.telemetry.precisionMode;

        faces.forEach((face) => {
          const applyScore = (eye: DetectedEye | null, keyId: string) => {
            if (!eye) return;
            const s = scoreMap.get(keyId);
            if (s) {
              eye.sharpness = s.sharpness;
              eye.sharpnessLabel = s.sharpnessLabel;
              eye.focusDiagnosis = s.diagnosis;
            }
          };
          applyScore(face.leftEye, `${face.id}_left`);
          applyScore(face.rightEye, `${face.id}_right`);

          const eyes = [face.leftEye, face.rightEye].filter((v): v is DetectedEye => v !== null);
          if (eyes.length > 0) {
            face.primaryEye = eyes.reduce((a, b) => (a.sharpness >= b.sharpness ? a : b));
            face.overallScore = Math.round(eyes.reduce((sum, e) => sum + e.sharpness, 0) / eyes.length);
          }
        });
      } catch (err) {
        // 非 Tauri 环境或调用失败时保持本地基础评分降级
        console.debug("NPU 人眼对焦大模型评估回退:", err);
      }
    }
  }

  const result: EyeDetectionResult = {
    photoId,
    photoPath: img.src,
    faces,
    detectionTimeMs: Math.round(performance.now() - start),
    computeDevice,
    precisionMode,
  };
  if (isCurrent()) {
    cache.set(key, result);
    while (cache.size > 24) cache.delete(cache.keys().next().value!);
  }
  return result;
}
