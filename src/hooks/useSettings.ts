import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { Settings } from "../types";

const defaultSettings: Settings = {
  theme: "dark",
  default_backup_count: 5,
  projects_directory: null,
  analysis_enabled: true,
  autosave_interval_seconds: 30,
};

export function useSettings() {
  const [settings, setSettings] = useState<Settings>(defaultSettings);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    invoke<Settings>("get_settings")
      .then(setSettings)
      .catch((err) => console.error("Failed to load settings:", err))
      .finally(() => setLoading(false));
  }, []);

  const updateSetting = useCallback(
    async (key: string, value: string) => {
      try {
        await invoke("update_setting", { key, value });
        setSettings((prev) => ({ ...prev, [key]: value }));
      } catch (err) {
        console.error("Failed to update setting:", err);
      }
    },
    [],
  );

  const resetSettings = useCallback(async () => {
    try {
      await invoke("reset_settings");
      setSettings(defaultSettings);
    } catch (err) {
      console.error("Failed to reset settings:", err);
    }
  }, []);

  return { settings, loading, updateSetting, resetSettings };
}
