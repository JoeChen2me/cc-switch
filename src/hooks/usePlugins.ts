import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import {
  pluginsApi,
  type InstalledPlugin,
  type AppType,
} from "@/lib/api/plugins";

export function useInstalledPlugins(appType: AppType) {
  return useQuery({
    queryKey: ["plugins", "installed", appType],
    queryFn: () => pluginsApi.scan(appType),
    staleTime: Infinity,
  });
}

export function useEnablePlugin(appType: AppType) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => pluginsApi.enable(id, appType),
    onSuccess: (updated) => {
      queryClient.setQueryData<InstalledPlugin[]>(
        ["plugins", "installed", appType],
        (old) =>
          old?.map((p) => (p.id === updated.id ? updated : p)) ?? [updated],
      );
    },
    onError: () => {
      queryClient.invalidateQueries({
        queryKey: ["plugins", "installed", appType],
      });
    },
  });
}

export function useDisablePlugin(appType: AppType) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => pluginsApi.disable(id, appType),
    onSuccess: (updated) => {
      queryClient.setQueryData<InstalledPlugin[]>(
        ["plugins", "installed", appType],
        (old) =>
          old?.map((p) => (p.id === updated.id ? updated : p)) ?? [updated],
      );
    },
    onError: () => {
      queryClient.invalidateQueries({
        queryKey: ["plugins", "installed", appType],
      });
    },
  });
}

export function useUninstallPlugin(appType: AppType) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => pluginsApi.uninstall(id, appType),
    onSuccess: (_, id) => {
      queryClient.setQueryData<InstalledPlugin[]>(
        ["plugins", "installed", appType],
        (old) => old?.filter((p) => p.id !== id),
      );
    },
    onError: () => {
      queryClient.invalidateQueries({
        queryKey: ["plugins", "installed", appType],
      });
    },
  });
}

export function useInstallPluginFromZip(appType: AppType) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (filePath: string) =>
      pluginsApi.installFromZip(filePath, appType),
    onSuccess: (installed) => {
      queryClient.setQueryData<InstalledPlugin[]>(
        ["plugins", "installed", appType],
        (old) => {
          if (!old) return installed;
          const existingIds = new Set(old.map((p) => p.id));
          const newPlugins = installed.filter((p) => !existingIds.has(p.id));
          return [...old, ...newPlugins].map((p) => {
            const updated = installed.find((u) => u.id === p.id);
            return updated ?? p;
          });
        },
      );
    },
    onError: () => {
      queryClient.invalidateQueries({
        queryKey: ["plugins", "installed", appType],
      });
    },
  });
}

export type { InstalledPlugin, AppType };
