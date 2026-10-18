<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch } from "vue";
import {
  Sparkles,
  AlertTriangle,
  Maximize2,
  Minimize2,
  Check,
  X,
  Star,
  Eye,
} from "lucide-vue-next";
import { usePhotoStore } from "../../stores/photoStore";
import { useI18n } from "../../i18n";
import type { ColorLabel, FlagStatus } from "../../types/photo";
import type { DetectedEye } from "../../types/eye";
import EyeInspectionPanel from "../EyeInspection/EyeInspectionPanel.vue";

const store = usePhotoStore();
const { t } = useI18n();
const canvasRef = ref<HTMLCanvasElement | null>(null);
const webglReady = ref(false);
const fallbackImageLoaded = ref(false);
const currentPreviewUrl = computed(() => store.getPreviewUrl(store.currentPhoto));

// 缩放平移与视口变换状态
const scale = ref(1.0);
const panX = ref(0);
const panY = ref(0);
const isDragging = ref(false);
const dragStart = ref({ x: 0, y: 0 });
const imageResolution = ref({ width: 1, height: 1 });
const is100Percent = ref(false);

// 着色器辅助特性开关
const showClippingWarning = ref(false);

// WebGL 状态
let gl: WebGL2RenderingContext | null = null;
let program: WebGLProgram | null = null;
let texture: WebGLTexture | null = null;
type GpuTextureEntry = { texture: WebGLTexture; width: number; height: number; pixelCount: number; usedAt: number };
const textureCache = new Map<string, GpuTextureEntry>();
const pendingTextureLoads = new Map<string, Promise<GpuTextureEntry | null>>();
const MAX_GPU_TEXTURES = 4;
const MAX_GPU_PIXELS = 64_000_000;
let positionBuffer: WebGLBuffer | null = null;
let texCoordBuffer: WebGLBuffer | null = null;
let animationFrameId: number | null = null;

// Vertex Shader (顶点着色器)
const vsSource = `#version 300 es
in vec2 a_position;
in vec2 a_texCoord;

uniform vec2 u_resolution;
uniform vec2 u_imageResolution;
uniform vec2 u_pan;
uniform float u_scale;

out vec2 v_texCoord;

void main() {
  // 根据视口大小与图片宽高比，计算居中自适应适配矩阵
  float aspectCanvas = u_resolution.x / u_resolution.y;
  float aspectImage = u_imageResolution.x / u_imageResolution.y;

  vec2 baseScale = vec2(1.0);
  if (aspectCanvas > aspectImage) {
    baseScale = vec2(aspectImage / aspectCanvas, 1.0);
  } else {
    baseScale = vec2(1.0, aspectCanvas / aspectImage);
  }

  // 计算缩放和平移变换
  vec2 pos = a_position * baseScale * u_scale;
  vec2 panNormalized = (u_pan / u_resolution) * 2.0;
  panNormalized.y = -panNormalized.y;
  
  gl_Position = vec4(pos + panNormalized, 0.0, 1.0);
  v_texCoord = a_texCoord;
}
`;

// Fragment Shader (片元着色器 - 包含 Sobel 峰值对焦与高光/暗部溢出预警)
const fsSource = `#version 300 es
precision highp float;

in vec2 v_texCoord;
uniform sampler2D u_image;
uniform vec2 u_imageResolution;
uniform bool u_peakingEnabled;
uniform bool u_clippingEnabled;

out vec4 fragColor;

// 亮度计算
float luma(vec3 color) {
  return dot(color, vec3(0.299, 0.587, 0.114));
}

void main() {
  vec4 color = texture(u_image, v_texCoord);
  vec2 onePixel = vec2(1.0) / u_imageResolution;

  // 1. Sobel 边缘检测峰值对焦 (Focus Peaking)
  if (u_peakingEnabled) {
    float p00 = luma(texture(u_image, v_texCoord + vec2(-onePixel.x, -onePixel.y)).rgb);
    float p10 = luma(texture(u_image, v_texCoord + vec2(0.0, -onePixel.y)).rgb);
    float p20 = luma(texture(u_image, v_texCoord + vec2(onePixel.x, -onePixel.y)).rgb);

    float p01 = luma(texture(u_image, v_texCoord + vec2(-onePixel.x, 0.0)).rgb);
    float p21 = luma(texture(u_image, v_texCoord + vec2(onePixel.x, 0.0)).rgb);

    float p02 = luma(texture(u_image, v_texCoord + vec2(-onePixel.x, onePixel.y)).rgb);
    float p12 = luma(texture(u_image, v_texCoord + vec2(0.0, onePixel.y)).rgb);
    float p22 = luma(texture(u_image, v_texCoord + vec2(onePixel.x, onePixel.y)).rgb);

    float gx = (p20 + 2.0 * p21 + p22) - (p00 + 2.0 * p01 + p02);
    float gy = (p02 + 2.0 * p12 + p22) - (p00 + 2.0 * p10 + p20);
    float edge = sqrt(gx * gx + gy * gy);

    if (edge > 0.12) {
      vec3 peakingColor = vec3(1.0, 0.18, 0.33); // 荧光鲜红
      color.rgb = mix(color.rgb, peakingColor, 0.85);
    }
  }

  // 2. 高光溢出 (过曝) 与 暗部死黑 (欠曝) 闪烁警示
  if (u_clippingEnabled) {
    if (color.r >= 0.995 || color.g >= 0.995 || color.b >= 0.995) {
      color.rgb = vec3(1.0, 0.0, 0.0); // 纯红高光溢出
    } else if (color.r <= 0.008 && color.g <= 0.008 && color.b <= 0.008) {
      color.rgb = vec3(0.0, 0.3, 1.0); // 纯蓝暗部死黑
    }
  }

  fragColor = color;
}
`;

function createShader(glCtx: WebGL2RenderingContext, type: number, source: string): WebGLShader | null {
  const shader = glCtx.createShader(type);
  if (!shader) return null;
  glCtx.shaderSource(shader, source);
  glCtx.compileShader(shader);
  if (!glCtx.getShaderParameter(shader, glCtx.COMPILE_STATUS)) {
    console.error("Shader 编译失败:", glCtx.getShaderInfoLog(shader));
    glCtx.deleteShader(shader);
    return null;
  }
  return shader;
}

function initWebGL() {
  if (!canvasRef.value) return;
  const canvas = canvasRef.value;
  gl = canvas.getContext("webgl2", { antialias: true, alpha: false });
  if (!gl) {
    console.warn("当前环境不支持 WebGL 2.0，将回退标准图片渲染");
    return;
  }

  const vs = createShader(gl, gl.VERTEX_SHADER, vsSource);
  const fs = createShader(gl, gl.FRAGMENT_SHADER, fsSource);
  if (!vs || !fs) return;

  program = gl.createProgram();
  if (!program) return;
  gl.attachShader(program, vs);
  gl.attachShader(program, fs);
  gl.linkProgram(program);

  if (!gl.getProgramParameter(program, gl.LINK_STATUS)) {
    console.error("WebGL 程序链接失败:", gl.getProgramInfoLog(program));
    return;
  }

  const positions = new Float32Array([
    -1, -1,
     1, -1,
    -1,  1,
    -1,  1,
     1, -1,
     1,  1,
  ]);
  positionBuffer = gl.createBuffer();
  gl.bindBuffer(gl.ARRAY_BUFFER, positionBuffer);
  gl.bufferData(gl.ARRAY_BUFFER, positions, gl.STATIC_DRAW);

  const texCoords = new Float32Array([
    0, 1,
    1, 1,
    0, 0,
    0, 0,
    1, 1,
    1, 0,
  ]);
  texCoordBuffer = gl.createBuffer();
  gl.bindBuffer(gl.ARRAY_BUFFER, texCoordBuffer);
  gl.bufferData(gl.ARRAY_BUFFER, texCoords, gl.STATIC_DRAW);
  webglReady.value = true;
}

let currentLoadingUrl = "";
let activeTextureUrl = "";
let preloadTimer: number | null = null;
let preloadGeneration = 0;
let disposed = false;

function getNativePixelScale() {
  if (!canvasRef.value) return 1;
  const canvas = canvasRef.value;
  const width = canvas.clientWidth * window.devicePixelRatio;
  const height = canvas.clientHeight * window.devicePixelRatio;
  if (width <= 0 || height <= 0) return 1;
  return Math.max(imageResolution.value.width / width, imageResolution.value.height / height);
}

function evictTextureCache() {
  if (!gl) return;
  const cachedPixels = () => [...textureCache.values()].reduce((sum, entry) => sum + entry.pixelCount, 0);
  while (textureCache.size > MAX_GPU_TEXTURES || cachedPixels() > MAX_GPU_PIXELS) {
    const oldest = [...textureCache.entries()]
      .filter(([url]) => url !== currentLoadingUrl && url !== activeTextureUrl)
      .sort((a, b) => a[1].usedAt - b[1].usedAt)[0];
    if (!oldest) break;
    gl.deleteTexture(oldest[1].texture);
    textureCache.delete(oldest[0]);
  }
}

function uploadTexture(url: string, img: HTMLImageElement): GpuTextureEntry | null {
  if (!gl || disposed) return null;
  const maxTextureSize = gl.getParameter(gl.MAX_TEXTURE_SIZE) as number;
  const maxEdgeForCache = Math.floor(Math.sqrt(MAX_GPU_PIXELS));
  const downscale = Math.min(1, maxTextureSize / Math.max(img.naturalWidth, img.naturalHeight), maxEdgeForCache / Math.max(img.naturalWidth, img.naturalHeight));
  let source: TexImageSource = img;
  let width = img.naturalWidth;
  let height = img.naturalHeight;
  if (downscale < 1) {
    width = Math.max(1, Math.floor(width * downscale));
    height = Math.max(1, Math.floor(height * downscale));
    const canvas = document.createElement("canvas");
    canvas.width = width;
    canvas.height = height;
    const context = canvas.getContext("2d");
    if (!context) return null;
    context.drawImage(img, 0, 0, width, height);
    source = canvas;
  }
  const uploaded = gl.createTexture();
  if (!uploaded) return null;
  gl.bindTexture(gl.TEXTURE_2D, uploaded);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.LINEAR);
  gl.pixelStorei(gl.UNPACK_FLIP_Y_WEBGL, false);
  gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, gl.RGBA, gl.UNSIGNED_BYTE, source);
  if (gl.getError() !== gl.NO_ERROR) {
    gl.deleteTexture(uploaded);
    return null;
  }
  textureCache.set(url, { texture: uploaded, width, height, pixelCount: width * height, usedAt: performance.now() });
  evictTextureCache();
  return textureCache.get(url) ?? null;
}

function requestTexture(url: string): Promise<GpuTextureEntry | null> {
  if (disposed) return Promise.resolve(null);
  const cached = textureCache.get(url);
  if (cached) {
    cached.usedAt = performance.now();
    return Promise.resolve(cached);
  }
  const pending = pendingTextureLoads.get(url);
  if (pending) return pending;
  if (!gl) return Promise.resolve(null);

  const img = new Image();
  img.crossOrigin = "anonymous";
  const loading = new Promise<GpuTextureEntry | null>((resolve) => {
    img.onload = () => resolve(uploadTexture(url, img));
    img.onerror = () => resolve(null);
  });
  pendingTextureLoads.set(url, loading);
  void loading.then(() => pendingTextureLoads.delete(url));
  img.src = url;
  return loading;
}

function activateTexture(url: string, entry: GpuTextureEntry) {
  if (!gl || currentLoadingUrl !== url) return;
  texture = entry.texture;
  activeTextureUrl = url;
  entry.usedAt = performance.now();
  imageResolution.value = { width: entry.width, height: entry.height };
  evictTextureCache();
  requestRender();
  queueNeighborPreload();
}

function loadTexture(url: string) {
  if (!gl || !url) return;
  if (preloadTimer !== null) window.clearTimeout(preloadTimer);
  preloadTimer = null;
  preloadGeneration++;
  currentLoadingUrl = url;
  requestRender();
  void requestTexture(url).then((entry) => {
    if (entry) activateTexture(url, entry);
    else if (currentLoadingUrl === url) console.error("[ViewportWebGL] 纹理加载失败:", url);
  });
}

function queueNeighborPreload() {
  if (preloadTimer !== null) window.clearTimeout(preloadTimer);
  const generation = ++preloadGeneration;
  preloadTimer = window.setTimeout(() => {
    preloadTimer = null;
    const list = store.filteredGroups;
    const current = store.currentPhoto;
    if (!current) return;
    const index = list.findIndex((photo) => photo.id === current.id);
    if (index < 0) return;
    const urls = [index + 1, index - 1, index + 2, index - 2]
      .filter((candidate) => candidate >= 0 && candidate < list.length)
      .map((candidate) => store.getPreviewUrl(list[candidate]));
    const run = async () => {
      for (const neighborUrl of urls) {
        if (generation !== preloadGeneration || !neighborUrl || neighborUrl === currentLoadingUrl) return;
        await requestTexture(neighborUrl);
      }
    };
    if ("requestIdleCallback" in window) {
      (window as Window & { requestIdleCallback: (cb: () => void, options?: { timeout: number }) => number }).requestIdleCallback(run, { timeout: 250 });
    } else run();
  }, 40);
}

function requestRender() {
  if (animationFrameId !== null) return;
  animationFrameId = requestAnimationFrame(render);
}

function render() {
  animationFrameId = null;
  if (!gl || !program || !canvasRef.value) return;

  const canvas = canvasRef.value;
  const displayWidth = canvas.clientWidth * window.devicePixelRatio;
  const displayHeight = canvas.clientHeight * window.devicePixelRatio;

  if (canvas.width !== displayWidth || canvas.height !== displayHeight) {
    canvas.width = displayWidth;
    canvas.height = displayHeight;
    gl.viewport(0, 0, canvas.width, canvas.height);
  }

  gl.clearColor(0.12, 0.12, 0.12, 1.0);
  gl.clear(gl.COLOR_BUFFER_BIT);
  if (!texture || activeTextureUrl !== currentLoadingUrl) return;

  gl.useProgram(program);

  const posLoc = gl.getAttribLocation(program, "a_position");
  gl.enableVertexAttribArray(posLoc);
  gl.bindBuffer(gl.ARRAY_BUFFER, positionBuffer);
  gl.vertexAttribPointer(posLoc, 2, gl.FLOAT, false, 0, 0);

  const texLoc = gl.getAttribLocation(program, "a_texCoord");
  gl.enableVertexAttribArray(texLoc);
  gl.bindBuffer(gl.ARRAY_BUFFER, texCoordBuffer);
  gl.vertexAttribPointer(texLoc, 2, gl.FLOAT, false, 0, 0);

  gl.uniform2f(gl.getUniformLocation(program, "u_resolution"), canvas.width, canvas.height);
  gl.uniform2f(
    gl.getUniformLocation(program, "u_imageResolution"),
    imageResolution.value.width,
    imageResolution.value.height
  );
  gl.uniform2f(gl.getUniformLocation(program, "u_pan"), panX.value * window.devicePixelRatio, panY.value * window.devicePixelRatio);
  gl.uniform1f(gl.getUniformLocation(program, "u_scale"), scale.value);
  gl.uniform1i(gl.getUniformLocation(program, "u_peakingEnabled"), store.focusPeaking ? 1 : 0);
  gl.uniform1i(gl.getUniformLocation(program, "u_clippingEnabled"), showClippingWarning.value ? 1 : 0);

  gl.activeTexture(gl.TEXTURE0);
  gl.bindTexture(gl.TEXTURE_2D, texture);
  gl.uniform1i(gl.getUniformLocation(program, "u_image"), 0);
  gl.drawArrays(gl.TRIANGLES, 0, 6);
}

// Preview URL also changes when switching between the paired RAW and JPG files.
watch(
  () => currentPreviewUrl.value,
  (url) => {
    scale.value = 1;
    panX.value = 0;
    panY.value = 0;
    is100Percent.value = false;
    const photo = store.currentPhoto;
    if (photo && url) {
      loadTexture(url);
      store.detectEyesForPhoto(photo);
    } else {
      currentLoadingUrl = "";
      activeTextureUrl = "";
      texture = null;
      imageResolution.value = { width: 1, height: 1 };
      store.detectEyesForPhoto(null);
      requestRender();
    }
  }
);

// 监听渲染开关
watch([() => store.focusPeaking, showClippingWarning, scale, panX, panY], () => {
  requestRender();
});

// 线性平滑插值缩放与平移 (Strictly Linear Interpolation)
let zoomAnimId: number | null = null;

function animateToTransform(destScale: number, destPanX: number, destPanY: number, durationMs = 140) {
  if (zoomAnimId !== null) {
    cancelAnimationFrame(zoomAnimId);
    zoomAnimId = null;
  }
  const startScale = scale.value;
  const startPanX = panX.value;
  const startPanY = panY.value;
  const startTime = performance.now();

  function step(now: number) {
    const elapsed = now - startTime;
    const progress = Math.min(1.0, elapsed / durationMs);
    
    scale.value = startScale + (destScale - startScale) * progress;
    panX.value = startPanX + (destPanX - startPanX) * progress;
    panY.value = startPanY + (destPanY - startPanY) * progress;
    requestRender();

    if (progress < 1.0) {
      zoomAnimId = requestAnimationFrame(step);
    } else {
      scale.value = destScale;
      panX.value = destPanX;
      panY.value = destPanY;
      zoomAnimId = null;
      requestRender();
    }
  }

  zoomAnimId = requestAnimationFrame(step);
}

// 缩放平移交互
const handleWheel = (e: WheelEvent) => {
  e.preventDefault();
  const zoomFactor = e.deltaY < 0 ? 1.15 : 0.85;
  const newScale = Math.max(0.1, Math.min(scale.value * zoomFactor, 6.0));
  animateToTransform(newScale, panX.value, panY.value, 110);
  is100Percent.value = false;
};

const handleMouseDown = (e: MouseEvent) => {
  if (zoomAnimId !== null) {
    cancelAnimationFrame(zoomAnimId);
    zoomAnimId = null;
  }
  isDragging.value = true;
  dragStart.value = { x: e.clientX - panX.value, y: e.clientY - panY.value };
};

const handleMouseMove = (e: MouseEvent) => {
  if (!isDragging.value) return;
  panX.value = e.clientX - dragStart.value.x;
  panY.value = e.clientY - dragStart.value.y;
};

const handleMouseUp = () => {
  isDragging.value = false;
};

// 空格键 / 按钮：一键切换 100% 局部放大与整幅适合屏幕 (线性过渡动画)
const toggle100Percent = () => {
  if (is100Percent.value || scale.value > 1.05) {
    animateToTransform(1.0, 0, 0, 160);
    is100Percent.value = false;
  } else {
    if (!canvasRef.value) return;
    const target = getNativePixelScale();
    animateToTransform(target, 0, 0, 160);
    is100Percent.value = true;
  }
};

const handleResetZoom = () => {
  animateToTransform(1.0, 0, 0, 160);
  is100Percent.value = false;
};

// 监听人眼智能吸附点击 (Smart Lock-On to Eye)
watch(
  () => store.targetLockOnEye,
  (target) => {
    if (target && canvasRef.value && imageResolution.value.width > 1 && imageResolution.value.height > 1) {
      const canvasW = canvasRef.value.clientWidth;
      const canvasH = canvasRef.value.clientHeight;
      const imgW = imageResolution.value.width;
      const imgH = imageResolution.value.height;

      // 100% 原始像素检查锐度，或至少 2.0x 放大倍率
      const destScale = Math.max(1.6, getNativePixelScale());

      const aspectCanvas = canvasW / canvasH;
      const aspectImage = imgW / imgH;
      let baseScaleX = 1.0;
      let baseScaleY = 1.0;
      if (aspectCanvas > aspectImage) {
        baseScaleX = aspectImage / aspectCanvas;
      } else {
        baseScaleY = aspectCanvas / aspectImage;
      }

      const quadX = (target.x - 0.5) * 2.0;
      const quadY = (0.5 - target.y) * 2.0;

      // 居中平移计算
      const destPanX = -(quadX * baseScaleX * destScale * canvasW) / 2.0;
      const destPanY = (quadY * baseScaleY * destScale * canvasH) / 2.0;

      animateToTransform(destScale, destPanX, destPanY, 180);
      is100Percent.value = true;
      store.targetLockOnEye = null;
    }
  }
);

// 实时计算已识别眼睛在当前视口屏幕中的投射像素位置
const eyeOverlays = computed(() => {
  if (!store.eyeAfEnabled || !canvasRef.value) return [];
  const canvasW = canvasRef.value.clientWidth;
  const canvasH = canvasRef.value.clientHeight;
  if (canvasW <= 0 || canvasH <= 0) return [];

  const aspectCanvas = canvasW / canvasH;
  const aspectImage = imageResolution.value.width / imageResolution.value.height;
  let baseScaleX = 1.0;
  let baseScaleY = 1.0;
  if (aspectCanvas > aspectImage) {
    baseScaleX = aspectImage / aspectCanvas;
  } else {
    baseScaleY = aspectCanvas / aspectImage;
  }

  const list: Array<{ id: string; x: number; y: number; sharpness: number; isOpen: boolean; eye: DetectedEye }> = [];

  for (const face of store.currentFaces) {
    const eyes = [face.leftEye, face.rightEye].filter(Boolean) as DetectedEye[];
    for (const eye of eyes) {
      const quadX = (eye.x - 0.5) * 2.0;
      const quadY = (0.5 - eye.y) * 2.0;

      const ndcX = quadX * baseScaleX * scale.value + (panX.value / canvasW) * 2.0;
      const ndcY = quadY * baseScaleY * scale.value - (panY.value / canvasH) * 2.0;

      const screenX = (ndcX + 1.0) * 0.5 * canvasW;
      const screenY = (1.0 - ndcY) * 0.5 * canvasH;

      if (screenX >= -30 && screenX <= canvasW + 30 && screenY >= -30 && screenY <= canvasH + 30) {
        list.push({
          id: `${face.id}_${eye.type}`,
          x: screenX,
          y: screenY,
          sharpness: eye.sharpness,
          isOpen: eye.isOpen,
          eye,
        });
      }
    }
  }
  return list;
});

const getBadgeLabel = (photo: any): string => {
  if (!photo) return "";
  if (photo.std_image && photo.raw_image) return "RAW+JPG";
  if (photo.raw_image) return "RAW";
  return "JPG";
};

onMounted(() => {
  disposed = false;
  initWebGL();
  if (store.currentPhoto) {
    fallbackImageLoaded.value = false;
    loadTexture(store.getPreviewUrl(store.currentPhoto));
    store.detectEyesForPhoto(store.currentPhoto);
  }
  window.addEventListener("resize", requestRender);
});

onUnmounted(() => {
  disposed = true;
  window.removeEventListener("resize", requestRender);
  if (preloadTimer !== null) window.clearTimeout(preloadTimer);
  preloadGeneration++;
  pendingTextureLoads.clear();
  for (const entry of textureCache.values()) gl?.deleteTexture(entry.texture);
  textureCache.clear();
  if (texture) gl?.deleteTexture(texture);
  if (program) gl?.deleteProgram(program);
  if (positionBuffer) gl?.deleteBuffer(positionBuffer);
  if (texCoordBuffer) gl?.deleteBuffer(texCoordBuffer);
  if (zoomAnimId !== null) cancelAnimationFrame(zoomAnimId);
  if (animationFrameId !== null) {
    cancelAnimationFrame(animationFrameId);
  }
});
</script>

<template>
  <div
    class="relative w-full h-full flex items-center justify-center overflow-hidden select-none cursor-grab active:cursor-grabbing"
    @wheel="handleWheel"
    @mousedown="handleMouseDown"
    @mousemove="handleMouseMove"
    @mouseup="handleMouseUp"
    @mouseleave="handleMouseUp"
    @dblclick="handleResetZoom"
  >
    <!-- WebGL 画布宿主 -->
    <canvas ref="canvasRef" class="w-full h-full block"></canvas>
    <img
      v-if="!webglReady && store.currentPhoto"
      :src="currentPreviewUrl"
      alt=""
      class="absolute inset-0 w-full h-full object-contain pointer-events-none"
      @load="fallbackImageLoaded = true"
      @error="fallbackImageLoaded = false"
    />
    <span
      v-if="!webglReady && !fallbackImageLoaded && store.currentPhoto"
      class="absolute inset-0 flex items-center justify-center text-xs text-[#8C8C8C] pointer-events-none"
    >
      {{ t.eyeAf.previewUnavailable }}
    </span>

    <!-- 相机取景器微单风格人眼识别追踪框 (Sony/Canon Eye AF Brackets) -->
    <div
      v-if="store.eyeAfEnabled"
      class="absolute inset-0 pointer-events-none overflow-hidden z-[15]"
    >
      <div
        v-for="eye in eyeOverlays"
        :key="eye.id"
        @click.stop="store.lockOnEye(eye.eye)"
        class="absolute -translate-x-1/2 -translate-y-1/2 flex items-center justify-center transition-all duration-75 pointer-events-auto cursor-pointer"
        :style="{
          left: `${eye.x}px`,
          top: `${eye.y}px`,
        }"
        :title="`${t.eyeAf.lockOnTip} (${eye.sharpness}%)`"
      >
        <div
          :class="[
            'w-7 h-7 rounded border-2 flex items-center justify-center shadow-lg transition-transform hover:scale-125',
            eye.isOpen
              ? eye.sharpness >= 75
                ? 'border-emerald-400 bg-emerald-500/10 shadow-emerald-500/30'
                : 'border-amber-400 bg-amber-500/10 shadow-amber-500/30'
              : 'border-rose-500 bg-rose-500/20 shadow-rose-500/30',
          ]"
        >
          <!-- 取景器准心 -->
          <div
            class="w-1.5 h-1.5 rounded-full"
            :class="eye.isOpen ? (eye.sharpness >= 75 ? 'bg-emerald-400' : 'bg-amber-400') : 'bg-rose-500'"
          />
        </div>
      </div>
    </div>

    <!-- 淘汰状态全屏黑遮罩 -->
    <div
      v-if="store.currentPhoto && store.currentPhoto.flag === -1"
      class="absolute inset-0 bg-black/60 z-10 flex items-center justify-center pointer-events-none"
    >
      <div class="text-center p-6 bg-black/80 rounded-xl border border-rose-500/50 backdrop-blur-md">
        <X class="w-16 h-16 text-rose-500 mx-auto stroke-[2.5]" />
        <span class="text-rose-400 font-bold text-xs tracking-wider uppercase mt-2 block">
          {{ t.loupe.rejectedOverlay }}
        </span>
      </div>
    </div>

    <!-- 左上角悬浮 HUD: 文件名 + RAW+JPG 徽章 + 缩放百分比 -->
    <div
      v-if="store.currentPhoto"
      class="absolute top-4 left-4 z-20 flex items-center space-x-2 px-3 py-1.5 rounded-lg bg-black/60 backdrop-blur-md border border-[#333333] shadow-lg"
    >
      <span class="font-mono font-bold text-xs text-[#E5E5E5]">
        {{ store.currentPhoto.base_name }}
      </span>
      <span
        :class="[
          'text-[10px] font-mono font-bold px-1.5 py-0.5 rounded shadow',
          store.currentPhoto.std_image && store.currentPhoto.raw_image
            ? 'bg-blue-600 text-white'
            : store.currentPhoto.raw_image
            ? 'bg-amber-600 text-white'
            : 'bg-zinc-700 text-zinc-200',
        ]"
      >
        {{ getBadgeLabel(store.currentPhoto) }}
      </span>
      <span class="text-[10px] font-mono text-[#8C8C8C] tabular-nums">
        {{ Math.round(scale * 100) }}%
      </span>
    </div>

    <!-- 右上角悬浮辅助工具条 (人眼对焦 / 峰值对焦 / 过曝警示 / 100% 切换) -->
    <div
      class="absolute top-4 right-4 z-20 flex items-center space-x-1.5 p-1 rounded-lg bg-black/60 backdrop-blur-md border border-[#333333] shadow-lg text-xs"
    >
      <!-- AI 人眼识别对焦追踪开关 -->
      <button
        @click="store.toggleEyeAf"
        :class="[
          'flex items-center space-x-1 px-2 py-1 rounded text-[11px] font-medium transition-colors cursor-pointer',
          store.eyeAfEnabled
            ? 'bg-sky-500/20 text-sky-400 border border-sky-500/30'
            : 'text-[#8C8C8C] hover:text-[#E5E5E5]',
        ]"
        :title="t.eyeAf.toggleOverlay"
      >
        <Eye class="w-3 h-3 text-sky-400" />
        <span>{{ t.eyeAf.toggleOverlay }}</span>
      </button>

      <!-- 峰值对焦切换 -->
      <button
        @click="store.focusPeaking = !store.focusPeaking"
        :class="[
          'flex items-center space-x-1 px-2 py-1 rounded text-[11px] font-medium transition-colors cursor-pointer',
          store.focusPeaking
            ? 'bg-rose-500/20 text-rose-400 border border-rose-500/30'
            : 'text-[#8C8C8C] hover:text-[#E5E5E5]',
        ]"
        :title="t.loupe.peaking"
      >
        <Sparkles class="w-3 h-3 text-rose-400" />
        <span>{{ t.loupe.peaking }}</span>
      </button>

      <!-- 过曝/欠曝闪烁警示 -->
      <button
        @click="showClippingWarning = !showClippingWarning"
        :class="[
          'flex items-center space-x-1 px-2 py-1 rounded text-[11px] font-medium transition-colors cursor-pointer',
          showClippingWarning
            ? 'bg-amber-500/20 text-amber-400 border border-amber-500/30'
            : 'text-[#8C8C8C] hover:text-[#E5E5E5]',
        ]"
        :title="t.loupe.clipping"
      >
        <AlertTriangle class="w-3 h-3 text-amber-400" />
        <span>{{ t.loupe.clipping }}</span>
      </button>

      <!-- 100% 检查锐度 -->
      <button
        @click="toggle100Percent"
        :class="[
          'flex items-center space-x-1 px-2 py-1 rounded text-[11px] font-medium transition-colors cursor-pointer',
          is100Percent
            ? 'bg-blue-500/20 text-blue-400 border border-blue-500/30'
            : 'text-[#8C8C8C] hover:text-[#E5E5E5]',
        ]"
        :title="is100Percent ? t.loupe.fitScreen : t.loupe.sharpness100"
      >
        <component :is="is100Percent ? Minimize2 : Maximize2" class="w-3 h-3" />
        <span>{{ is100Percent ? t.loupe.fitScreen : t.loupe.sharpness100 }}</span>
      </button>
    </div>

    <!-- Narrative Select 风格人眼特写检视浮动面板 -->
    <EyeInspectionPanel v-if="store.eyeAfEnabled && store.currentPhoto" />

    <!-- 底部中央悬浮打标条 -->
    <div
      v-if="store.currentPhoto"
      class="absolute bottom-4 z-20 flex items-center space-x-3 px-4 py-2 rounded-xl bg-black/70 backdrop-blur-md border border-[#333333] shadow-2xl"
    >
      <div class="flex items-center space-x-1">
        <button
          @click="store.setFlag(1 as FlagStatus)"
          :class="[
            'flex items-center space-x-1 px-2.5 py-1 rounded text-xs font-semibold transition-all cursor-pointer',
            store.currentPhoto.flag === 1
              ? 'bg-emerald-500 text-white shadow-lg shadow-emerald-500/30'
              : 'bg-[#262626] text-[#8C8C8C] hover:text-[#E5E5E5]',
          ]"
          :title="t.loupe.pickKey"
        >
          <Check class="w-3.5 h-3.5 stroke-[3]" />
          <span>{{ t.loupe.pickKey }}</span>
        </button>
        <button
          @click="store.setFlag(-1 as FlagStatus)"
          :class="[
            'flex items-center space-x-1 px-2.5 py-1 rounded text-xs font-semibold transition-all cursor-pointer',
            store.currentPhoto.flag === -1
              ? 'bg-rose-500 text-white shadow-lg shadow-rose-500/30'
              : 'bg-[#262626] text-[#8C8C8C] hover:text-[#E5E5E5]',
          ]"
          :title="t.loupe.rejectKey"
        >
          <X class="w-3.5 h-3.5 stroke-[3]" />
          <span>{{ t.loupe.rejectKey }}</span>
        </button>
      </div>

      <div class="h-4 w-[1px] bg-[#333333]"></div>

      <!-- 星级打分点阵 -->
      <div class="flex items-center space-x-1">
        <button
          v-for="star in 5"
          :key="star"
          @click="store.setRating(store.currentPhoto?.rating === star ? 0 : star)"
          class="p-1 hover:scale-125 transition-transform cursor-pointer"
          :title="t.loupe.rateStar.replace('{0}', star.toString())"
        >
          <Star
            :class="[
              'w-4 h-4 transition-colors',
              (store.currentPhoto?.rating || 0) >= star
                ? 'text-amber-400 fill-amber-400'
                : 'text-[#444444] hover:text-[#8C8C8C]',
            ]"
          />
        </button>
      </div>

      <div class="h-4 w-[1px] bg-[#333333]"></div>

      <!-- 快捷色标 -->
      <div class="flex items-center space-x-1.5">
        <button
          v-for="color in (['red', 'yellow', 'green', 'blue', 'purple'] as ColorLabel[])"
          :key="color"
          @click="store.setColorLabel(color)"
          :class="[
            'w-4 h-4 rounded-full transition-transform',
            store.currentPhoto?.color_label === color
              ? 'ring-2 ring-white scale-125'
              : 'opacity-70 hover:opacity-100',
            color === 'red' ? 'bg-[#EF4444]' : '',
            color === 'yellow' ? 'bg-[#F59E0B]' : '',
            color === 'green' ? 'bg-[#10B981]' : '',
            color === 'blue' ? 'bg-[#3B82F6]' : '',
            color === 'purple' ? 'bg-[#8B5CF6]' : '',
          ]"
        />
      </div>
    </div>
  </div>
</template>
