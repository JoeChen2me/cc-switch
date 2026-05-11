import React, { useState } from "react";
import { useTranslation } from "react-i18next";
import { Loader2, Power, PowerOff, Trash2, Upload, FileJson } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { ListItemRow } from "@/components/common/ListItemRow";
import { ConfirmDialog } from "@/components/ConfirmDialog";
import { toast } from "sonner";
import { skillsApi } from "@/lib/api";
import {
  useInstalledPlugins,
  useEnablePlugin,
  useDisablePlugin,
  useUninstallPlugin,
  useInstallPluginFromZip,
  type InstalledPlugin,
} from "@/hooks/usePlugins";

interface PluginsPanelProps {
  onRefresh?: () => void;
}

export function PluginsPanel({ onRefresh }: PluginsPanelProps) {
  const { t } = useTranslation();
  const [confirmDialog, setConfirmDialog] = useState<{
    isOpen: boolean;
    title: string;
    message: string;
    variant?: "destructive" | "info";
    onConfirm: () => void;
  } | null>(null);
  const [detailPlugin, setDetailPlugin] = useState<InstalledPlugin | null>(null);
  const [togglingId, setTogglingId] = useState<string | null>(null);

  const { data: plugins, isLoading, refetch } = useInstalledPlugins();
  const enableMutation = useEnablePlugin();
  const disableMutation = useDisablePlugin();
  const uninstallMutation = useUninstallPlugin();
  const zipInstallMutation = useInstallPluginFromZip();

  const handleRefresh = async () => {
    await refetch();
    onRefresh?.();
  };

  const handleToggle = async (plugin: InstalledPlugin) => {
    setTogglingId(plugin.id);
    try {
      if (plugin.enabled) {
        await disableMutation.mutateAsync(plugin.id);
        toast.success(t("plugins.disabled", { name: plugin.name }));
      } else {
        await enableMutation.mutateAsync(plugin.id);
        toast.success(t("plugins.enabled", { name: plugin.name }));
      }
    } catch (error) {
      toast.error(t("common.error"), { description: String(error) });
    } finally {
      setTogglingId(null);
    }
  };

  const handleUninstall = (plugin: InstalledPlugin) => {
    setConfirmDialog({
      isOpen: true,
      title: t("plugins.uninstall"),
      message: t("plugins.uninstallConfirm", { name: plugin.name }),
      variant: "destructive",
      onConfirm: async () => {
        try {
          await uninstallMutation.mutateAsync(plugin.id);
          setConfirmDialog(null);
          toast.success(t("plugins.uninstallSuccess", { name: plugin.name }));
        } catch (error) {
          toast.error(t("common.error"), { description: String(error) });
        }
      },
    });
  };

  const handleInstallFromZip = async () => {
    try {
      const filePath = await skillsApi.openZipFileDialog();
      if (!filePath) return;
      const installed = await zipInstallMutation.mutateAsync(filePath);
      toast.success(t("plugins.zipInstallSuccess", { count: installed.length }));
    } catch (error) {
      toast.error(t("common.error"), { description: String(error) });
    }
  };

  const handleOpenManifest = async (plugin: InstalledPlugin) => {
    setDetailPlugin(detailPlugin?.id === plugin.id ? null : plugin);
  };

  if (isLoading) {
    return (
      <div className="flex items-center justify-center py-12">
        <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
      </div>
    );
  }

  const enabledCount = plugins?.filter((p) => p.enabled).length ?? 0;
  const disabledCount = plugins?.filter((p) => !p.enabled).length ?? 0;

  return (
    <div className="flex flex-col h-full">
      {/* 工具栏 */}
      <div className="flex items-center justify-between px-4 py-2 border-b border-border-default">
        <div className="flex items-center gap-2 text-sm text-muted-foreground">
          <span>{t("plugins.total", { total: plugins?.length ?? 0 })}</span>
          <Badge variant="default" className="bg-green-600/90 text-white border-0 text-[10px] px-1.5 py-0 h-4">
            {enabledCount} {t("plugins.enabledLabel")}
          </Badge>
          {disabledCount > 0 && (
            <Badge variant="secondary" className="text-[10px] px-1.5 py-0 h-4">
              {disabledCount} {t("plugins.disabledLabel")}
            </Badge>
          )}
        </div>
        <div className="flex items-center gap-1">
          <Button variant="ghost" size="sm" onClick={handleInstallFromZip} disabled={zipInstallMutation.isPending}>
            {zipInstallMutation.isPending ? (
              <Loader2 className="h-3.5 w-3.5 mr-1.5 animate-spin" />
            ) : (
              <Upload className="h-3.5 w-3.5 mr-1.5" />
            )}
            {t("plugins.installZip")}
          </Button>
          <Button variant="ghost" size="sm" onClick={handleRefresh}>
            {t("common.refresh")}
          </Button>
        </div>
      </div>

      {/* 列表 */}
      <div className="flex-1 overflow-y-auto">
        {!plugins || plugins.length === 0 ? (
          <div className="flex flex-col items-center justify-center py-16 text-muted-foreground">
            <p className="text-sm">{t("plugins.empty")}</p>
          </div>
        ) : (
          plugins.map((plugin, index) => (
            <React.Fragment key={plugin.id}>
              <ListItemRow isLast={index === plugins.length - 1 && detailPlugin?.id !== plugin.id}>
                {/* 名称和描述 */}
                <div className="flex-1 min-w-0">
                  <div className="flex items-center gap-2">
                    <span className="font-medium text-sm truncate">{plugin.name}</span>
                    {plugin.version && (
                      <Badge variant="outline" className="text-[10px] px-1.5 py-0 h-4 shrink-0">
                        v{plugin.version}
                      </Badge>
                    )}
                    {!plugin.enabled && (
                      <Badge variant="secondary" className="text-[10px] px-1.5 py-0 h-4 shrink-0">
                        {t("plugins.disabledLabel")}
                      </Badge>
                    )}
                  </div>
                  {plugin.description && (
                    <p className="text-xs text-muted-foreground truncate mt-0.5">{plugin.description}</p>
                  )}
                  <div className="flex items-center gap-2 mt-0.5">
                    {plugin.author && (
                      <span className="text-[10px] text-muted-foreground/70">{plugin.author}</span>
                    )}
                  </div>
                </div>

                {/* 操作按钮 */}
                <div className="flex items-center gap-1 shrink-0">
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={() => handleOpenManifest(plugin)}
                    title={t("plugins.viewManifest")}
                  >
                    <FileJson className="h-3.5 w-3.5" />
                  </Button>
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={() => handleToggle(plugin)}
                    disabled={togglingId !== null && togglingId !== plugin.id}
                    title={plugin.enabled ? t("plugins.disable") : t("plugins.enable")}
                  >
                    {plugin.enabled ? (
                      <PowerOff className="h-3.5 w-3.5 text-orange-500" />
                    ) : (
                      <Power className="h-3.5 w-3.5 text-green-500" />
                    )}
                  </Button>
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={() => handleUninstall(plugin)}
                    disabled={uninstallMutation.isPending || togglingId !== null}
                    title={t("plugins.uninstall")}
                  >
                    <Trash2 className="h-3.5 w-3.5 text-red-500" />
                  </Button>
                </div>
              </ListItemRow>

              {/* plugin.json 详情展开 */}
              {detailPlugin?.id === plugin.id && plugin.pluginJsonRaw && (
                <div className="px-4 py-2 border-b border-border-default bg-muted/30">
                  <pre className="text-xs text-muted-foreground overflow-x-auto whitespace-pre-wrap break-all max-h-48 overflow-y-auto">
                    {(() => {
                      try {
                        return JSON.stringify(JSON.parse(plugin.pluginJsonRaw!), null, 2);
                      } catch {
                        return plugin.pluginJsonRaw;
                      }
                    })()}
                  </pre>
                </div>
              )}
            </React.Fragment>
          ))
        )}
      </div>

      {/* 确认对话框 */}
      {confirmDialog && (
        <ConfirmDialog
          isOpen={confirmDialog.isOpen}
          title={confirmDialog.title}
          message={confirmDialog.message}
          variant={confirmDialog.variant}
          onConfirm={confirmDialog.onConfirm}
          onCancel={() => setConfirmDialog(null)}
        />
      )}
    </div>
  );
}
