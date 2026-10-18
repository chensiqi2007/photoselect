export interface DetectedEye {
  type: "left" | "right";
  // 归一化坐标 (0.0 ~ 1.0)
  x: number;
  y: number;
  // 裁切区域像素绝对范围
  box: {
    x: number;
    y: number;
    width: number;
    height: number;
  };
  // 焦点清晰度得分 (0 ~ 100)
  sharpness: number;
  sharpnessLabel: "sharp" | "good" | "soft";
  // 睁眼/眨眼检测
  isOpen: boolean;
  opennessRatio: number; // 0.0 ~ 1.0
  // 100% 裁切高分辨率预览图 (Base64 DataURL)
  cropDataUrl: string;
}

export interface DetectedFace {
  id: number;
  // 人脸归一化中心与尺寸
  centerX: number;
  centerY: number;
  box: {
    x: number;
    y: number;
    width: number;
    height: number;
  };
  // 100% 人脸裁切缩略图 (Base64 DataURL)
  faceCropDataUrl: string;
  // 双眼检测实体
  leftEye: DetectedEye | null;
  rightEye: DetectedEye | null;
  // 综合判定 (对焦是否扎实，是否闭眼)
  primaryEye: DetectedEye | null;
  overallScore: number;
  isBlinking: boolean;
}

export interface EyeDetectionResult {
  photoId: string;
  photoPath: string;
  faces: DetectedFace[];
  detectionTimeMs: number;
}
