import { invoke } from "@tauri-apps/api/core";

/** 已安装的 Plugin（Claude Code 或 Codex） */
export interface InstalledPlugin {
  id: string;
  appType: "claude" | "codex";
  name: string;
  version?: string;
  description?: string;
  author?: string;
  directoryName: string;
  enabled: boolean;
  installedAt: number;
  contentHash?: string;
  pluginJsonRaw?: string;
}

export type AppType = "claude" | "codex";

export const pluginsApi = {
  /** 扫描指定应用的所有已安装 Plugin */
  async scan(appType: AppType): Promise<InstalledPlugin[]> {
    return await invoke("scan_plugins", { appType });
  },

  /** 启用 Plugin */
  async enable(id: string, appType: AppType): Promise<InstalledPlugin> {
    return await invoke("enable_plugin", { id, appType });
  },

  /** 禁用 Plugin */
  async disable(id: string, appType: AppType): Promise<InstalledPlugin> {
    return await invoke("disable_plugin", { id, appType });
  },

  /** 卸载 Plugin */
  async uninstall(id: string, appType: AppType): Promise<boolean> {
    return await invoke("uninstall_plugin", { id, appType });
  },

  /** 从 ZIP 文件安装 Plugin */
  async installFromZip(
    filePath: string,
    appType: AppType,
  ): Promise<InstalledPlugin[]> {
    return await invoke("install_plugin_from_zip", { filePath, appType });
  },
};
