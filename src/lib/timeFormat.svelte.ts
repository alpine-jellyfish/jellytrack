import { getPreference, setPreference } from "./db";

export const TIME_FORMAT_PREF_KEY = "time_format";

export const timeFormat = $state({ use24h: true });

export async function loadTimeFormat(): Promise<void> {
  const v = await getPreference(TIME_FORMAT_PREF_KEY);
  timeFormat.use24h = v !== "ampm";
}

export async function setTimeFormat(use24h: boolean): Promise<void> {
  timeFormat.use24h = use24h;
  await setPreference(TIME_FORMAT_PREF_KEY, use24h ? "24h" : "ampm");
}
