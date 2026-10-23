import { ref, computed } from "vue";

export type Locale = "zh-CN" | "zh-TW" | "en-US";

export interface Translations {
  topbar: {
    openFolder: string;
    scanning: string;
    noFolder: string;
    loupe: string;
    grid: string;
    compare: string;
    autoAdvance: string;
    autoAdvanceTip: string;
    filterMinRating: string;
    pickFilter: string;
    rejectFilter: string;
    rawPlusJpg: string;
    rawPreview: string;
    export: string;
    settings: string;
    aiSelection: string;
  };
  sidebar: {
    storageDrives: string;
    smartCollections: string;
    allPhotos: string;
    picked: string;
    fiveStar: string;
    rejected: string;
    unflagged: string;
    colorLabels: string;
    recentWorkspaces: string;
    red: string;
    yellow: string;
    green: string;
    blue: string;
    purple: string;
  };
  grid: {
    totalPhotos: string;
    selectedIndex: string;
    compareTarget: string;
    hints: string;
    noPhotos: string;
    resetGrid: string;
    refBadge: string;
    candBadge: string;
  };
  compare: {
    refPhoto: string;
    candPhoto: string;
    swap: string;
    pickARejectB: string;
    pickBRejectA: string;
    syncZoom: string;
    indepZoom: string;
    fitScreen: string;
    sharpness100: string;
    noRef: string;
    noCand: string;
    compareTip: string;
    availablePhotos: string;
    rejectedBadge: string;
    pick: string;
    reject: string;
  };
  loupe: {
    peaking: string;
    clipping: string;
    rejectedOverlay: string;
    pickKey: string;
    rejectKey: string;
    rateStar: string;
    fitScreen: string;
    sharpness100: string;
  };
  inspector: {
    exifTitle: string;
    aperture: string;
    shutter: string;
    iso: string;
    focal: string;
    cameraModel: string;
    lensModel: string;
    captureTime: string;
    fileSize: string;
    bindingStatus: string;
    hotkeysTitle: string;
    rawPlusJpgBinding: string;
    rawOnly: string;
    stdOnly: string;
    keysPrevNext: string;
    keysRate: string;
    keysPick: string;
    keysReject: string;
    keysColor: string;
    keysMode: string;
    keysPeaking: string;
    keysBg: string;
  };
  settings: {
    title: string;
    appearance: string;
    themeDark: string;
    themeDarkDesc: string;
    themeLight: string;
    themeLightDesc: string;
    viewportBg: string;
    bgNeutral: string;
    bgBlack: string;
    bgWhite: string;
    language: string;
    langZhCn: string;
    langZhTw: string;
    langEnUs: string;
    uiScale: string;
    uiScaleDesc: string;
    about: string;
    version: string;
    engineDesc: string;
    close: string;
    aiTitle: string;
    aiEnabled: string;
    aiBaseUrl: string;
    aiModel: string;
    aiApiKey: string;
    aiKeySaved: string;
    aiKeyMissing: string;
    aiSave: string;
    aiTest: string;
    aiClearKey: string;
    aiPrivacy: string;
    aiVisionHint: string;
  };
  aiSelection: {
    title: string;
    run: string;
    running: string;
    close: string;
    noResults: string;
    progress: string;
    score: string;
    pick: string;
    review: string;
    reject: string;
    applySelected: string;
    choose: string;
    applyRating: string;
    applyFlag: string;
    viewPhoto: string;
    configure: string;
    selectAll: string;
    countLimit: string;
    confirmed: string;
  };
  export: {
    title: string;
    pickedOnly: string;
    filteredOnly: string;
    allPhotos: string;
    selectDir: string;
    startExport: string;
  };
  eyeAf: {
    title: string;
    toggleOverlay: string;
    togglePanel: string;
    detecting: string;
    detectionFailed: string;
    previewUnavailable: string;
    noFaces: string;
    person: string;
    leftEye: string;
    rightEye: string;
    sharpFocus: string;
    goodFocus: string;
    softFocus: string;
    eyeOpen: string;
    eyeBlink: string;
    lockOnZoom: string;
    lockOnTip: string;
    settingsEyeAf: string;
    settingsEyeAfDesc: string;
  };
}

const zhCN: Translations = {
  topbar: {
    openFolder: "打开目录",
    scanning: "秒级扫描中...",
    noFolder: "未挂载任何文件夹",
    loupe: "单图 [E]",
    grid: "网格 [G]",
    compare: "对比 [C]",
    autoAdvance: "自动推进",
    autoAdvanceTip: "打分或标记旗标后，自动切至下一张照片 (开启后效率翻倍)",
    filterMinRating: "仅显示 >= {0} 星的照片",
    pickFilter: "筛选留用照片 [Pick]",
    rejectFilter: "筛选淘汰照片 [Reject]",
    rawPlusJpg: "RAW+JPG",
    rawPreview: "切换 RAW / JPG 预览",
    export: "导出",
    settings: "系统设置",
    aiSelection: "AI 选图",
  },
  aiSelection: {
    title: "AI 选图建议",
    run: "分析当前筛选照片",
    running: "正在分析…",
    close: "关闭",
    noResults: "暂无建议。运行分析后逐张复核。",
    progress: "已完成 {0} / {1} 张",
    score: "评分",
    pick: "留用建议",
    review: "人工复核",
    reject: "淘汰建议",
    applySelected: "应用已勾选字段",
    choose: "选择此建议",
    applyRating: "应用评分",
    applyFlag: "应用旗标",
    viewPhoto: "查看照片",
    configure: "配置 AI 服务",
    selectAll: "选择全部建议",
    countLimit: "一次最多分析 500 张，请先筛选或分批操作。",
    confirmed: "已应用人工确认的建议",
  },
  sidebar: {
    storageDrives: "存储介质 / 读卡器",
    smartCollections: "智能分类视图",
    allPhotos: "全部照片",
    picked: "已留用 (Picked)",
    fiveStar: "五星精选",
    rejected: "已淘汰 (Rejected)",
    unflagged: "未标记",
    colorLabels: "色彩标记",
    recentWorkspaces: "最近打开目录",
    red: "红色",
    yellow: "黄色",
    green: "绿色",
    blue: "蓝色",
    purple: "紫色",
  },
  grid: {
    totalPhotos: "共 {0} 张照片",
    selectedIndex: "当前选定: #{0}",
    compareTarget: "对比图: #{0}",
    hints: "💡 按住 Ctrl 点击照片选择两张对比 • Ctrl+滚轮缩放 • 双击大图",
    noPhotos: "当前筛选条件下无照片",
    resetGrid: "重置默认大小 (220px)",
    refBadge: "A (参考)",
    candBadge: "B (对比)",
  },
  compare: {
    refPhoto: "图 A (参考)",
    candPhoto: "图 B (对比)",
    swap: "互换 [S]",
    pickARejectB: "留用 A · 淘汰 B",
    pickBRejectA: "留用 B · 淘汰 A",
    syncZoom: "联动同步",
    indepZoom: "独立缩放",
    fitScreen: "适应屏幕",
    sharpness100: "100% 锐度",
    noRef: "暂无参考照片",
    noCand: "暂无对比照片 (按住 Ctrl 点击其他照片即可选定)",
    compareTip: "提示: 按住 Ctrl 点击胶片栏或网格照片，可随时更换对比目标照片；按 S 互换两张位置；按 ← / → 顺序浏览候选照片",
    availablePhotos: "对比视图: {0} 张照片可用",
    rejectedBadge: "已淘汰",
    pick: "留用",
    reject: "淘汰",
  },
  loupe: {
    peaking: "峰值 [F]",
    clipping: "过曝警示",
    rejectedOverlay: "已标记为淘汰 (Rejected)",
    pickKey: "留用 [P]",
    rejectKey: "淘汰 [X]",
    rateStar: "打分 {0} 星 [数字键 {0}]",
    fitScreen: "适应屏幕",
    sharpness100: "100% 锐度",
  },
  inspector: {
    exifTitle: "拍摄参数 (EXIF)",
    aperture: "光圈:",
    shutter: "快门:",
    iso: "ISO:",
    focal: "焦距:",
    cameraModel: "机身型号:",
    lensModel: "镜头型号:",
    captureTime: "拍摄时间:",
    fileSize: "文件大小:",
    bindingStatus: "绑定状态:",
    hotkeysTitle: "全键盘暗房流速查",
    rawPlusJpgBinding: "RAW+JPG (同名绑定)",
    rawOnly: "纯 RAW",
    stdOnly: "纯标准图 (JPG/PNG)",
    keysPrevNext: "A / D 上下张",
    keysRate: "0 - 5 快速打星",
    keysPick: "P 留用 (Pick)",
    keysReject: "X 淘汰 (Reject)",
    keysColor: "6 - 9 色标分类",
    keysMode: "E/G/C 切换模式",
    keysPeaking: "F 峰值对焦",
    keysBg: "B 切换背景",
  },
  settings: {
    title: "系统设置",
    appearance: "外观与色彩主题",
    themeDark: "黑暗模式 (暗房标准)",
    themeDarkDesc: "18% 标准摄影中性灰基准，减少视觉疲劳，真实呈现色彩与动态范围",
    themeLight: "白昼模式 (明亮工作室)",
    themeLightDesc: "高对比度清爽界面，适合明亮光线环境与办公室快速阅览",
    viewportBg: "单图视口背景基准色",
    bgNeutral: "18% 中性灰",
    bgBlack: "纯黑 #000",
    bgWhite: "纯白 #FFF",
    language: "界面语言",
    langZhCn: "简体中文",
    langZhTw: "繁體中文",
    langEnUs: "English",
    uiScale: "页面与字体缩放",
    uiScaleDesc: "自适应 High-DPI 高分屏，支持 100%、125%、150%、200% 页面与字体比例缩放",
    about: "关于 PhotoSelect Desktop",
    version: "版本",
    engineDesc: "Zero-Import 零导入纯原生相机 RAW+JPG 极速选片平台 v1.1.0。全新搭载高速连拍聚合、感知哈希严格去重与同主体分组折叠视图，支持 XMP 生态级双向同步、100%~200% High-DPI 缩放与 AI 智能选片。",
    close: "完成",
    aiTitle: "AI 选图服务",
    aiEnabled: "启用视觉 AI 选图",
    aiBaseUrl: "兼容 OpenAI 的 API Base URL",
    aiModel: "视觉模型名称",
    aiApiKey: "API Key（留空则保留已保存密钥）",
    aiKeySaved: "密钥已保存至系统凭据库",
    aiKeyMissing: "尚未保存 API Key",
    aiSave: "保存配置",
    aiTest: "测试连接",
    aiClearKey: "删除已保存密钥",
    aiPrivacy: "分析时会将最长边 768px 的照片预览发送至此服务，不会上传 RAW 原文件。AI 结果只作为建议，需人工确认。",
    aiVisionHint: "请使用支持图像输入与 Chat Completions 的模型。",
  },
  export: {
    title: "类 LrC 快捷导出照片",
    pickedOnly: "仅导出留用照片",
    filteredOnly: "导出当前筛选照片",
    allPhotos: "导出全部照片",
    selectDir: "选择导出目标路径",
    startExport: "立即执行导出",
  },
  eyeAf: {
    title: "AI 人眼对焦与神态特写",
    toggleOverlay: "人眼追踪框 [O]",
    togglePanel: "特写检视面板",
    detecting: "正在毫秒级识别人眼与对焦...",
    detectionFailed: "人眼检测不可用，请检查本地模型与 GPU/CPU 运行环境",
    previewUnavailable: "无法载入照片预览",
    noFaces: "当前画面未检测到明显人脸与双眼",
    person: "人物",
    leftEye: "左眼",
    rightEye: "右眼",
    sharpFocus: "焦点极锐",
    goodFocus: "对焦良好",
    softFocus: "对焦偏软/脱焦",
    eyeOpen: "睁眼",
    eyeBlink: "闭眼/眨眼",
    lockOnZoom: "吸附对焦",
    lockOnTip: "点击任意眼部特写，视口将以 100% 原始像素丝滑吸附居中至该人眼",
    settingsEyeAf: "自动人眼识别与对焦质检",
    settingsEyeAfDesc: "自动扫描画面中的人脸、双眼焦点锐度与闭眼神态，并在特写面板中呈现 100% 裁切",
  },
};

const zhTW: Translations = {
  topbar: {
    openFolder: "開啟目錄",
    scanning: "秒級掃描中...",
    noFolder: "未掛載任何資料夾",
    loupe: "單圖 [E]",
    grid: "網格 [G]",
    compare: "對比 [C]",
    autoAdvance: "自動推進",
    autoAdvanceTip: "評分或標記旗標後，自動切換至下一張照片 (開啟後效率翻倍)",
    filterMinRating: "僅顯示 >= {0} 星的照片",
    pickFilter: "篩選留用照片 [Pick]",
    rejectFilter: "篩選淘汰照片 [Reject]",
    rawPlusJpg: "RAW+JPG",
    rawPreview: "切換 RAW / JPG 預覽",
    export: "導出",
    settings: "系統設定",
    aiSelection: "AI 選圖",
  },
  aiSelection: {
    title: "AI 選圖建議",
    run: "分析目前篩選照片",
    running: "正在分析…",
    close: "關閉",
    noResults: "暫無建議。執行分析後逐張複核。",
    progress: "已完成 {0} / {1} 張",
    score: "評分",
    pick: "留用建議",
    review: "人工複核",
    reject: "淘汰建議",
    applySelected: "套用已勾選欄位",
    choose: "選擇此建議",
    applyRating: "套用評分",
    applyFlag: "套用旗標",
    viewPhoto: "檢視照片",
    configure: "設定 AI 服務",
    selectAll: "選擇全部建議",
    countLimit: "一次最多分析 500 張，請先篩選或分批操作。",
    confirmed: "已套用人工確認的建議",
  },
  sidebar: {
    storageDrives: "存儲介質 / 讀卡器",
    smartCollections: "智慧分類檢視",
    allPhotos: "全部照片",
    picked: "已留用 (Picked)",
    fiveStar: "五星精選",
    rejected: "已淘汰 (Rejected)",
    unflagged: "未標記",
    colorLabels: "色彩標記",
    recentWorkspaces: "最近開啟目錄",
    red: "紅色",
    yellow: "黃色",
    green: "綠色",
    blue: "藍色",
    purple: "紫色",
  },
  grid: {
    totalPhotos: "共 {0} 張照片",
    selectedIndex: "目前選定: #{0}",
    compareTarget: "對比圖: #{0}",
    hints: "💡 按住 Ctrl 點擊照片選擇兩張對比 • Ctrl+滾輪縮放 • 雙擊大圖",
    noPhotos: "目前篩選條件下無照片",
    resetGrid: "重置預設大小 (220px)",
    refBadge: "A (參考)",
    candBadge: "B (對比)",
  },
  compare: {
    refPhoto: "圖 A (參考)",
    candPhoto: "圖 B (對比)",
    swap: "互換 [S]",
    pickARejectB: "留用 A · 淘汰 B",
    pickBRejectA: "留用 B · 淘汰 A",
    syncZoom: "聯動同步",
    indepZoom: "獨立縮放",
    fitScreen: "適應螢幕",
    sharpness100: "100% 銳度",
    noRef: "暫無參考照片",
    noCand: "暫無對比照片 (按住 Ctrl 點擊其他照片即可選定)",
    compareTip: "提示: 按住 Ctrl 點擊膠片欄或網格照片，可隨時更換對比目標照片；按 S 互換兩張位置；按 ← / → 順序瀏覽候選照片",
    availablePhotos: "對比檢視: {0} 張照片可用",
    rejectedBadge: "已淘汰",
    pick: "留用",
    reject: "淘汰",
  },
  loupe: {
    peaking: "峰值 [F]",
    clipping: "過曝警示",
    rejectedOverlay: "已標記為淘汰 (Rejected)",
    pickKey: "留用 [P]",
    rejectKey: "淘汰 [X]",
    rateStar: "評分 {0} 星 [數字鍵 {0}]",
    fitScreen: "適應螢幕",
    sharpness100: "100% 銳度",
  },
  inspector: {
    exifTitle: "拍攝參數 (EXIF)",
    aperture: "光圈:",
    shutter: "快門:",
    iso: "ISO:",
    focal: "焦距:",
    cameraModel: "機身型號:",
    lensModel: "鏡頭型號:",
    captureTime: "拍攝時間:",
    fileSize: "檔案大小:",
    bindingStatus: "綁定狀態:",
    hotkeysTitle: "全鍵盤暗房流速查",
    rawPlusJpgBinding: "RAW+JPG (同名綁定)",
    rawOnly: "純 RAW",
    stdOnly: "純標準圖 (JPG/PNG)",
    keysPrevNext: "A / D 上下張",
    keysRate: "0 - 5 快速打星",
    keysPick: "P 留用 (Pick)",
    keysReject: "X 淘汰 (Reject)",
    keysColor: "6 - 9 色標分類",
    keysMode: "E/G/C 切換模式",
    keysPeaking: "F 峰值對焦",
    keysBg: "B 切換背景",
  },
  settings: {
    title: "系統設定",
    appearance: "外觀與色彩主題",
    themeDark: "黑暗模式 (暗房標準)",
    themeDarkDesc: "18% 標準攝影中性灰基準，減少視覺疲勞，真實呈現色彩與動態範圍",
    themeLight: "白晝模式 (明亮工作室)",
    themeLightDesc: "高對比度清爽介面，適合明亮光線環境與辦公室快速閱覽",
    viewportBg: "單圖視口背景基準色",
    bgNeutral: "18% 中性灰",
    bgBlack: "純黑 #000",
    bgWhite: "純白 #FFF",
    language: "介面語言",
    langZhCn: "簡體中文",
    langZhTw: "繁體中文",
    langEnUs: "English",
    uiScale: "頁面與字型縮放",
    uiScaleDesc: "自適應 High-DPI 高解析度螢幕，支援 100%、125%、150%、200% 介面與字型比例縮放",
    about: "關於 PhotoSelect Desktop",
    version: "版本",
    engineDesc: "Zero-Import 零導入純原生相機 RAW+JPG 極速選片平台 v1.1.0。全新搭載高速連拍聚合、感知雜湊嚴格去重與同主體分組折疊檢視，支援 XMP 生態級雙向同步、100%~200% High-DPI 縮放與 AI 智慧選片。",
    close: "完成",
    aiTitle: "AI 選圖服務",
    aiEnabled: "啟用視覺 AI 選圖",
    aiBaseUrl: "相容 OpenAI 的 API Base URL",
    aiModel: "視覺模型名稱",
    aiApiKey: "API Key（留空則保留已儲存密鑰）",
    aiKeySaved: "密鑰已儲存至系統憑據庫",
    aiKeyMissing: "尚未儲存 API Key",
    aiSave: "儲存設定",
    aiTest: "測試連線",
    aiClearKey: "刪除已儲存密鑰",
    aiPrivacy: "分析時會將最長邊 768px 的照片預覽傳送至此服務，不會上傳 RAW 原始檔。AI 結果僅為建議，需人工確認。",
    aiVisionHint: "請使用支援影像輸入與 Chat Completions 的模型。",
  },
  export: {
    title: "類 LrC 快捷導出照片",
    pickedOnly: "僅導出留用照片",
    filteredOnly: "導出目前篩選照片",
    allPhotos: "導出全部照片",
    selectDir: "選擇導出目標路徑",
    startExport: "立即執行導出",
  },
  eyeAf: {
    title: "AI 人眼對焦與神態特寫",
    toggleOverlay: "人眼追蹤框 [O]",
    togglePanel: "特寫檢視面板",
    detecting: "正在毫秒級辨識人眼與對焦...",
    detectionFailed: "人眼偵測無法使用，請檢查本機模型與 GPU/CPU 執行環境",
    previewUnavailable: "無法載入照片預覽",
    noFaces: "目前畫面未偵測到明顯人臉與雙眼",
    person: "人物",
    leftEye: "左眼",
    rightEye: "右眼",
    sharpFocus: "焦點極銳",
    goodFocus: "對焦良好",
    softFocus: "對焦偏軟/脫焦",
    eyeOpen: "睜眼",
    eyeBlink: "閉眼/眨眼",
    lockOnZoom: "吸附對焦",
    lockOnTip: "點擊任意眼部特寫，視口將以 100% 原始像素絲滑吸附居中至該人眼",
    settingsEyeAf: "自動人眼辨識與對焦質檢",
    settingsEyeAfDesc: "自動掃描畫面中的人臉、雙眼焦點銳度與閉眼神態，並在特寫面板中呈現 100% 裁切",
  },
};

const enUS: Translations = {
  topbar: {
    openFolder: "Open Folder",
    scanning: "Scanning...",
    noFolder: "No folder mounted",
    loupe: "Loupe [E]",
    grid: "Grid [G]",
    compare: "Compare [C]",
    autoAdvance: "Auto-Advance",
    autoAdvanceTip: "Automatically advance to next photo after rating or flagging",
    filterMinRating: "Show >= {0} stars",
    pickFilter: "Filter Picked [P]",
    rejectFilter: "Filter Rejected [X]",
    rawPlusJpg: "RAW+JPG",
    rawPreview: "Toggle RAW / JPG preview",
    export: "Export",
    settings: "Settings",
    aiSelection: "AI selection",
  },
  aiSelection: {
    title: "AI selection suggestions",
    run: "Analyze filtered photos",
    running: "Analyzing…",
    close: "Close",
    noResults: "No suggestions yet. Run analysis and review each photo.",
    progress: "Completed {0} / {1}",
    score: "Score",
    pick: "Pick suggestion",
    review: "Review",
    reject: "Reject suggestion",
    applySelected: "Apply checked fields",
    choose: "Select this suggestion",
    applyRating: "Apply rating",
    applyFlag: "Apply flag",
    viewPhoto: "View photo",
    configure: "Configure AI service",
    selectAll: "Select all suggestions",
    countLimit: "Analyze up to 500 photos per run. Filter or split the selection.",
    confirmed: "Human-confirmed suggestions applied",
  },
  sidebar: {
    storageDrives: "Storage / Card Readers",
    smartCollections: "Smart Collections",
    allPhotos: "All Photos",
    picked: "Picked",
    fiveStar: "5-Star Favorites",
    rejected: "Rejected",
    unflagged: "Unflagged",
    colorLabels: "Color Labels",
    recentWorkspaces: "Recent Workspaces",
    red: "Red",
    yellow: "Yellow",
    green: "Green",
    blue: "Blue",
    purple: "Purple",
  },
  grid: {
    totalPhotos: "{0} Photos",
    selectedIndex: "Selected: #{0}",
    compareTarget: "Compare: #{0}",
    hints: "💡 Hold Ctrl & click to compare • Ctrl+Wheel to resize • Double click for loupe",
    noPhotos: "No photos match filter",
    resetGrid: "Reset Size (220px)",
    refBadge: "A (Ref)",
    candBadge: "B (Cand)",
  },
  compare: {
    refPhoto: "Photo A (Ref)",
    candPhoto: "Photo B (Cand)",
    swap: "Swap [S]",
    pickARejectB: "Pick A · Reject B",
    pickBRejectA: "Pick B · Reject A",
    syncZoom: "Sync Zoom",
    indepZoom: "Separate Zoom",
    fitScreen: "Fit Screen",
    sharpness100: "100% Zoom",
    noRef: "No reference photo",
    noCand: "No candidate photo (Hold Ctrl and click any photo to pair)",
    compareTip: "Tip: Hold Ctrl and click in filmstrip/grid to pair; press S to swap; press ← / → to cycle candidate",
    availablePhotos: "Compare View: {0} photos available",
    rejectedBadge: "Rejected",
    pick: "Pick",
    reject: "Reject",
  },
  loupe: {
    peaking: "Peaking [F]",
    clipping: "Clipping Alert",
    rejectedOverlay: "Marked as Rejected",
    pickKey: "Pick [P]",
    rejectKey: "Reject [X]",
    rateStar: "Rate {0} Stars [{0}]",
    fitScreen: "Fit Screen",
    sharpness100: "100% Sharpness",
  },
  inspector: {
    exifTitle: "Camera EXIF",
    aperture: "Aperture:",
    shutter: "Shutter:",
    iso: "ISO:",
    focal: "Focal:",
    cameraModel: "Camera:",
    lensModel: "Lens:",
    captureTime: "Date & Time:",
    fileSize: "File Size:",
    bindingStatus: "Binding:",
    hotkeysTitle: "Keyboard Shortcuts",
    rawPlusJpgBinding: "RAW+JPG (Paired)",
    rawOnly: "RAW Only",
    stdOnly: "JPG/PNG Only",
    keysPrevNext: "A / D Prev/Next",
    keysRate: "0 - 5 Star Rating",
    keysPick: "P Pick",
    keysReject: "X Reject",
    keysColor: "6 - 9 Color Label",
    keysMode: "E/G/C View Mode",
    keysPeaking: "F Focus Peaking",
    keysBg: "B Background Grey",
  },
  settings: {
    title: "System Settings",
    appearance: "Appearance & Theme",
    themeDark: "Dark Mode (Darkroom Neutral)",
    themeDarkDesc: "18% standard photography neutral grey baseline, eliminates eye strain, true color rendering",
    themeLight: "Light Mode (Studio Clean)",
    themeLightDesc: "Clean studio white layout, ideal for daytime use and office environments",
    viewportBg: "Viewport Background Grey",
    bgNeutral: "18% Neutral Grey",
    bgBlack: "Pure Black #000",
    bgWhite: "Pure White #FFF",
    language: "Interface Language",
    langZhCn: "简体中文 (Simplified Chinese)",
    langZhTw: "繁體中文 (Traditional Chinese)",
    langEnUs: "English (US)",
    uiScale: "UI & Font Scaling",
    uiScaleDesc: "Adaptive High-DPI display scaling, supporting 100%, 125%, 150%, and 200%",
    about: "About PhotoSelect Desktop",
    version: "Version",
    engineDesc: "Zero-Import native RAW+JPG ultra-fast culling platform v1.1.0. Featuring burst clustering, dHash duplicate detection & subject-group folded views, bidirectional XMP keyword sync, 100%-200% High-DPI scaling, and AI-assisted culling.",
    close: "Done",
    aiTitle: "AI Photo Selection",
    aiEnabled: "Enable vision AI selection",
    aiBaseUrl: "OpenAI-compatible API Base URL",
    aiModel: "Vision model",
    aiApiKey: "API key (leave blank to keep saved key)",
    aiKeySaved: "Key saved in the OS credential store",
    aiKeyMissing: "No API key saved",
    aiSave: "Save settings",
    aiTest: "Test connection",
    aiClearKey: "Delete saved key",
    aiPrivacy: "Analysis sends photo previews up to 768px on the longest edge to this service. RAW originals are not uploaded. AI output is only a suggestion and requires your confirmation.",
    aiVisionHint: "Use a model that supports image input and Chat Completions.",
  },
  export: {
    title: "Export Photos",
    pickedOnly: "Export Picked Only",
    filteredOnly: "Export Filtered Photos",
    allPhotos: "Export All Photos",
    selectDir: "Select Destination Directory",
    startExport: "Execute Export",
  },
  eyeAf: {
    title: "AI Eye AF & Focus Inspector",
    toggleOverlay: "Eye AF Overlay [O]",
    togglePanel: "Close-Ups Panel",
    detecting: "Analyzing eyes and focus...",
    detectionFailed: "Eye detection is unavailable. Check the local model and GPU/CPU runtime.",
    previewUnavailable: "Photo preview could not be loaded.",
    noFaces: "No clear faces or eyes detected",
    person: "Person",
    leftEye: "Left Eye",
    rightEye: "Right Eye",
    sharpFocus: "Tack Sharp",
    goodFocus: "Good Focus",
    softFocus: "Soft / Missed Focus",
    eyeOpen: "Open",
    eyeBlink: "Blinked",
    lockOnZoom: "Lock-On",
    lockOnTip: "Click any eye crop to smoothly lock on and center the viewport at 100% pixel zoom",
    settingsEyeAf: "Automatic Eye Detection & Focus Inspection",
    settingsEyeAfDesc: "Automatically detect faces, eye sharpness, and blink status with 100% crop inspection loupe",
  },
};

const dictionaries: Record<Locale, Translations> = {
  "zh-CN": zhCN,
  "zh-TW": zhTW,
  "en-US": enUS,
};

// 反应式当前语言，初始化自 localStorage
const initialLocale = (typeof localStorage !== "undefined" && (localStorage.getItem("photoselect_lang") as Locale)) || "zh-CN";
const currentLocale = ref<Locale>(initialLocale in dictionaries ? initialLocale : "zh-CN");

export function useI18n() {
  const t = computed(() => dictionaries[currentLocale.value] || zhCN);

  const setLocale = (loc: Locale) => {
    if (loc in dictionaries) {
      currentLocale.value = loc;
      try {
        localStorage.setItem("photoselect_lang", loc);
      } catch {
        // ignore
      }
    }
  };

  return {
    locale: currentLocale,
    t,
    setLocale,
  };
}
