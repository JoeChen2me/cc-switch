import { invoke } from "@tauri-apps/api/core";

/** 已安装的 Claude Code Plugin */
export interface InstalledPlugin {
  id: string;
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

export const pluginsApi = {
  /** 扫描所有已安装的 Plugin（同步文件系统与数据库） */
  async scan(): Promise<InstalledPlugin[]> {
    return await invoke("scan_plugins");
  },

  /** 启用 Plugin（从 disabled 目录移回 plugins 目录） */
  async enable(id: string): Promise<InstalledPlugin> {
    return await invoke("enable_plugin", { id });
  },

  /** 禁用 Plugin（从 plugins 目录移到 disabled 目录） */
  async disable(id: string): Promise<InstalledPlugin> {
    return await invoke("disable_plugin", { id });
  },

  /** 卸载 Plugin（彻底删除） */
  async uninstall(id: string): Promise<boolean> {
    return await invoke("uninstall_plugin", { id });
  },

  /** 从 ZIP 文件安装 Plugin */
  async installFromZip(filePath: string): Promise<InstalledPlugin[]> {
    return await invoke("install_plugin_from_zip", { filePath });
  },
};
