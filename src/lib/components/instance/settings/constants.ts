export const NAV_OSCPUS = [
  "Windows NT 10.0; Win64; x64",
  "Intel Mac OS X 10.15",
  "Intel Mac OS X 10_15_7",
  "Linux x86_64",
];
export const NAV_BUILD_IDS = ["20181001000000", "20100101"];
export const NAV_APP_NAMES = ["Netscape"];
export const NAV_PRODUCT_SUBS = ["20030107", "20100101"];

export const PLATFORMS = [
  "Win32",
  "MacIntel",
  "Linux x86_64",
  "iPhone",
  "Linux armv81",
  "Linux aarch64",
];

export const CONCURRENCY_OPTS = [1, 2, 4, 6, 8, 10, 12, 16, 20, 24, 32, 64];

export const TOUCH_POINTS = [0, 1, 2, 5, 10];

export const SCREEN_PRESETS = [
  { label: "1280 × 720 (HD)", w: 1280, h: 720 },
  { label: "1366 × 768 (WXGA)", w: 1366, h: 768 },
  { label: "1440 × 900 (WXGA+)", w: 1440, h: 900 },
  { label: "1600 × 900 (HD+)", w: 1600, h: 900 },
  { label: "1920 × 1080 (FHD)", w: 1920, h: 1080 },
  { label: "2048 × 1152", w: 2048, h: 1152 },
  { label: "2560 × 1080 (UW-FHD)", w: 2560, h: 1080 },
  { label: "2560 × 1440 (QHD)", w: 2560, h: 1440 },
  { label: "2560 × 1600 (WQXGA)", w: 2560, h: 1600 },
  { label: "3440 × 1440 (UW-QHD)", w: 3440, h: 1440 },
  { label: "3840 × 2160 (4K)", w: 3840, h: 2160 },
  { label: "5120 × 2880 (5K)", w: 5120, h: 2880 },
  { label: "1024 × 768 (XGA)", w: 1024, h: 768 },
  { label: "1280 × 800 (WXGA)", w: 1280, h: 800 },
  { label: "1280 × 1024 (SXGA)", w: 1280, h: 1024 },
  { label: "1680 × 1050 (WSXGA+)", w: 1680, h: 1050 },
];

export const COLOR_DEPTHS = [16, 24, 30, 32];

export const DPR_OPTS = [0.75, 1.0, 1.25, 1.5, 1.75, 2.0, 2.25, 2.5, 3.0, 3.5, 4.0];

export const SAMPLE_RATES = [
  8000, 11025, 22050, 44100, 48000, 88200, 96000, 176400, 192000,
];

export const MAX_CHANNELS = [1, 2, 4, 6, 8, 16, 32, 64, 128, 256];

export const WEBGL_PRESETS: { label: string; renderer: string; vendor: string }[] = [
  { label: "AUTO", renderer: "", vendor: "" },
  { label: "Apple M1", renderer: "Apple M1, or similar", vendor: "Apple" },
  { label: "Apple M2", renderer: "Apple M2, or similar", vendor: "Apple" },
  { label: "Apple M3", renderer: "Apple M3, or similar", vendor: "Apple" },
  {
    label: "NVIDIA GTX 980 (ANGLE)",
    renderer: "ANGLE (NVIDIA, NVIDIA GeForce GTX 980 Direct3D11 vs_5_0 ps_5_0), or similar",
    vendor: "Google Inc. (NVIDIA)",
  },
  {
    label: "NVIDIA GTX 1050 (ANGLE)",
    renderer: "ANGLE (NVIDIA, NVIDIA GeForce GTX 1050 Ti Direct3D11 vs_5_0 ps_5_0), or similar",
    vendor: "Google Inc. (NVIDIA)",
  },
  {
    label: "NVIDIA GTX 1080 (ANGLE)",
    renderer: "ANGLE (NVIDIA, NVIDIA GeForce GTX 1080 Direct3D11 vs_5_0 ps_5_0), or similar",
    vendor: "Google Inc. (NVIDIA)",
  },
  {
    label: "NVIDIA RTX 2060 (ANGLE)",
    renderer: "ANGLE (NVIDIA, NVIDIA GeForce RTX 2060 Direct3D11 vs_5_0 ps_5_0), or similar",
    vendor: "Google Inc. (NVIDIA)",
  },
  {
    label: "NVIDIA RTX 3070 (ANGLE)",
    renderer: "ANGLE (NVIDIA, NVIDIA GeForce RTX 3070 Direct3D11 vs_5_0 ps_5_0), or similar",
    vendor: "Google Inc. (NVIDIA)",
  },
  {
    label: "NVIDIA RTX 3080 (ANGLE)",
    renderer: "ANGLE (NVIDIA, NVIDIA GeForce RTX 3080 Direct3D11 vs_5_0 ps_5_0), or similar",
    vendor: "Google Inc. (NVIDIA)",
  },
  {
    label: "NVIDIA RTX 4080 (ANGLE)",
    renderer: "ANGLE (NVIDIA, NVIDIA GeForce RTX 4080 Direct3D11 vs_5_0 ps_5_0), or similar",
    vendor: "Google Inc. (NVIDIA)",
  },
  {
    label: "AMD RX 580 (ANGLE)",
    renderer: "ANGLE (AMD, Radeon RX 580 Series Direct3D11 vs_5_0 ps_5_0), or similar",
    vendor: "Google Inc. (AMD)",
  },
  {
    label: "AMD RX 6700 XT (ANGLE)",
    renderer: "ANGLE (AMD, AMD Radeon RX 6700 XT Direct3D11 vs_5_0 ps_5_0), or similar",
    vendor: "Google Inc. (AMD)",
  },
  {
    label: "Intel UHD 620 (ANGLE)",
    renderer: "ANGLE (Intel, Intel(R) UHD Graphics 620 Direct3D11 vs_5_0 ps_5_0), or similar",
    vendor: "Google Inc. (Intel)",
  },
  {
    label: "Intel Iris Xe (ANGLE)",
    renderer: "ANGLE (Intel, Intel(R) Iris(R) Xe Graphics Direct3D11 vs_5_0 ps_5_0), or similar",
    vendor: "Google Inc. (Intel)",
  },
  {
    label: "Intel 945GM (ANGLE)",
    renderer: "ANGLE (Intel, Intel 945GM Direct3D11 vs_4_0 ps_4_0)",
    vendor: "Google Inc. (Intel)",
  },
  {
    label: "GeForce GTX 980 (Mesa)",
    renderer: "GeForce GTX 980, or similar",
    vendor: "Mesa",
  },
  { label: "Custom…", renderer: "", vendor: "" },
];

export const LOCALES: { label: string; lang: string; region: string }[] = [
  { label: "AUTO", lang: "", region: "" },
  { label: "English (US)", lang: "en", region: "US" },
  { label: "English (GB)", lang: "en", region: "GB" },
  { label: "English (AU)", lang: "en", region: "AU" },
  { label: "English (CA)", lang: "en", region: "CA" },
  { label: "French (FR)", lang: "fr", region: "FR" },
  { label: "French (CA)", lang: "fr", region: "CA" },
  { label: "German (DE)", lang: "de", region: "DE" },
  { label: "Spanish (ES)", lang: "es", region: "ES" },
  { label: "Spanish (MX)", lang: "es", region: "MX" },
  { label: "Portuguese (BR)", lang: "pt", region: "BR" },
  { label: "Portuguese (PT)", lang: "pt", region: "PT" },
  { label: "Italian (IT)", lang: "it", region: "IT" },
  { label: "Dutch (NL)", lang: "nl", region: "NL" },
  { label: "Russian (RU)", lang: "ru", region: "RU" },
  { label: "Chinese Simplified (CN)", lang: "zh", region: "CN" },
  { label: "Chinese Traditional (TW)", lang: "zh", region: "TW" },
  { label: "Japanese (JP)", lang: "ja", region: "JP" },
  { label: "Korean (KR)", lang: "ko", region: "KR" },
  { label: "Arabic (SA)", lang: "ar", region: "SA" },
  { label: "Polish (PL)", lang: "pl", region: "PL" },
  { label: "Turkish (TR)", lang: "tr", region: "TR" },
  { label: "Swedish (SE)", lang: "sv", region: "SE" },
  { label: "Norwegian (NO)", lang: "nb", region: "NO" },
  { label: "Danish (DK)", lang: "da", region: "DK" },
  { label: "Finnish (FI)", lang: "fi", region: "FI" },
  { label: "Indonesian (ID)", lang: "id", region: "ID" },
  { label: "Hindi (IN)", lang: "hi", region: "IN" },
  { label: "Vietnamese (VN)", lang: "vi", region: "VN" },
  { label: "Thai (TH)", lang: "th", region: "TH" },
  { label: "Custom…", lang: "", region: "" },
];

export const WINDOW_PRESETS = [
  { label: "1024 × 768", w: 1024, h: 768 },
  { label: "1280 × 720 (HD)", w: 1280, h: 720 },
  { label: "1280 × 800", w: 1280, h: 800 },
  { label: "1366 × 768", w: 1366, h: 768 },
  { label: "1440 × 900", w: 1440, h: 900 },
  { label: "1536 × 864", w: 1536, h: 864 },
  { label: "1600 × 900", w: 1600, h: 900 },
  { label: "1680 × 1050", w: 1680, h: 1050 },
  { label: "1920 × 1080 (FHD)", w: 1920, h: 1080 },
  { label: "2560 × 1440 (QHD)", w: 2560, h: 1440 },
  { label: "3840 × 2160 (4K)", w: 3840, h: 2160 },
];

export const MEDIA_COUNTS = [0, 1, 2, 3, 4];

export let timezones: string[] = [];
try {
  timezones = (Intl as any).supportedValuesOf("timeZone");
} catch {
  timezones = [
    "America/New_York",
    "America/Chicago",
    "America/Denver",
    "America/Los_Angeles",
    "America/Sao_Paulo",
    "America/Toronto",
    "America/Vancouver",
    "America/Mexico_City",
    "Europe/London",
    "Europe/Paris",
    "Europe/Berlin",
    "Europe/Madrid",
    "Europe/Rome",
    "Europe/Amsterdam",
    "Europe/Warsaw",
    "Europe/Moscow",
    "Europe/Istanbul",
    "Asia/Tokyo",
    "Asia/Shanghai",
    "Asia/Hong_Kong",
    "Asia/Seoul",
    "Asia/Singapore",
    "Asia/Dubai",
    "Asia/Kolkata",
    "Asia/Bangkok",
    "Asia/Jakarta",
    "Australia/Sydney",
    "Australia/Melbourne",
    "Pacific/Auckland",
    "Africa/Cairo",
    "Africa/Johannesburg",
    "Africa/Lagos",
  ];
}
