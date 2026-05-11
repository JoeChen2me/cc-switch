import {
  useMutation,
  useQuery,
  useQueryClient,
  keepPreviousData,
} from "@tanstack/react-query";
import {
  pluginsApi,
  type InstalledPlugin,
} from "@/lib/api/plugins";

export function useInstalledPlugins() {
  return useQuery({
    queryKey: ["plugins", "installed"],
    queryFn: () => pluginsApi.scan(),
    staleTime: Infinity,
    placeholderData: keepPreviousData,
  });
}

export function useEnablePlugin() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => pluginsApi.enable(id),
    onSuccess: (updated) => {
      queryClient.setQueryData<InstalledPlugin[]>(
        ["plugins", "installed"],
        (old) =>
          old?.map((p) => (p.id === updated.id ? updated : p)) ?? [updated],
      );
    },
    onError: () => {
      queryClient.invalidateQueries({ queryKey: ["plugins", "installed"] });
    },
  });
}

export function useDisablePlugin() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => pluginsApi.disable(id),
    onSuccess: (updated) => {
      queryClient.setQueryData<InstalledPlugin[]>(
        ["plugins", "installed"],
        (old) =>
          old?.map((p) => (p.id === updated.id ? updated : p)) ?? [updated],
      );
    },
    onError: () => {
      queryClient.invalidateQueries({ queryKey: ["plugins", "installed"] });
    },
  });
}

export function useUninstallPlugin() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => pluginsApi.uninstall(id),
    onSuccess: (_, id) => {
      queryClient.setQueryData<InstalledPlugin[]>(
        ["plugins", "installed"],
        (old) => old?.filter((p) => p.id !== id),
      );
    },
    onError: () => {
      queryClient.invalidateQueries({ queryKey: ["plugins", "installed"] });
    },
  });
}

export function useInstallPluginFromZip() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (filePath: string) => pluginsApi.installFromZip(filePath),
    onSuccess: (installed) => {
      queryClient.setQueryData<InstalledPlugin[]>(
        ["plugins", "installed"],
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
      queryClient.invalidateQueries({ queryKey: ["plugins", "installed"] });
    },
  });
}

export type { InstalledPlugin };
