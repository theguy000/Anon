import type { FingerprintConfig } from "$lib/store";

export function numVal(v: number | null | undefined): string {
  return v != null ? String(v) : "";
}

export function setNum(fp: FingerprintConfig, field: keyof FingerprintConfig, e: Event): FingerprintConfig {
  const val = (e.target as HTMLInputElement).value;
  (fp as any)[field] = val === "" ? null : Number(val);
  return fp;
}

export function setFloat(fp: FingerprintConfig, field: keyof FingerprintConfig, e: Event): FingerprintConfig {
  const val = (e.target as HTMLInputElement).value;
  (fp as any)[field] = val === "" ? null : parseFloat(val);
  return fp;
}

export function setStr(fp: FingerprintConfig, field: keyof FingerprintConfig, e: Event): FingerprintConfig {
  const val = (e.target as HTMLInputElement).value;
  (fp as any)[field] = val === "" ? null : val;
  return fp;
}

export function setBool(fp: FingerprintConfig, field: keyof FingerprintConfig, val: boolean | null): FingerprintConfig {
  (fp as any)[field] = val;
  return fp;
}

export function setSelectStr(fp: FingerprintConfig, field: keyof FingerprintConfig, e: Event): FingerprintConfig {
  const val = (e.target as HTMLSelectElement).value;
  (fp as any)[field] = val === "" ? null : val;
  return fp;
}

export function setSelectNum(fp: FingerprintConfig, field: keyof FingerprintConfig, e: Event): FingerprintConfig {
  const val = (e.target as HTMLSelectElement).value;
  (fp as any)[field] = val === "" ? null : Number(val);
  return fp;
}

export function setSelectFloat(fp: FingerprintConfig, field: keyof FingerprintConfig, e: Event): FingerprintConfig {
  const val = (e.target as HTMLSelectElement).value;
  (fp as any)[field] = val === "" ? null : parseFloat(val);
  return fp;
}
