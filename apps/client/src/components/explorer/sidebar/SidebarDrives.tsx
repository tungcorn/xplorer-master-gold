import React, { useState, useEffect, useCallback } from 'react';
import { HardDrive, User, ArrowUpFromLine } from 'lucide-react';
import { TauriAPI } from '@/lib/tauri-api';
import { isWindows, ROOT_PATH } from '@/lib/constants';
import { useToast } from '@/hooks/use-toast';
import { useTranslation } from 'react-i18next';

interface Drive {
  letter: string;
  label: string;
  path: string;
  total_space: number;
  free_space: number;
}

interface SidebarDrivesProps {
  navigateToPath: (path: string) => void;
}

const SidebarDrives = ({ navigateToPath }: SidebarDrivesProps) => {
  const { t } = useTranslation();
  const { toast } = useToast();
  const [drives, setDrives] = useState<Drive[]>([]);
  const [homePath, setHomePath] = useState<string | null>(null);

  useEffect(() => {
    TauriAPI.listDrives()
      .then((list) => setDrives(list))
      .catch((error) => {
        console.error('Failed to load drives:', error);
        setDrives([
          {
            letter: isWindows ? 'C' : '',
            label: isWindows ? 'Local Disk' : 'Macintosh HD',
            path: ROOT_PATH,
            total_space: 0,
            free_space: 0,
          },
        ]);
      });

    if (!isWindows) {
      TauriAPI.getUserDirectories()
        .then((dirs) => setHomePath(dirs.home))
        .catch((error) => console.error('Failed to load user directories:', error));
    }
  }, []);

  const handleEjectVolume = useCallback(
    async (path: string, e: React.MouseEvent) => {
      e.stopPropagation();
      try {
        await TauriAPI.ejectVolume(path);
        toast({ title: t('drives.ejectSuccess') });
        const driveList = await TauriAPI.listDrives();
        setDrives(driveList);
      } catch (error) {
        toast({
          title: t('drives.ejectFailed'),
          description: String(error),
          variant: 'destructive',
        });
        console.error('Eject volume failed:', error);
      }
    },
    [t, toast],
  );

  return (
    <div
      role="region"
      aria-label={isWindows ? 'Drives' : 'Volumes'}
      className="space-y-1 px-3 py-2"
    >
      {drives.map((drive) => {
        const totalGB =
          drive.total_space > 0 ? Math.round(drive.total_space / (1024 * 1024 * 1024)) : 0;
        const freeGB =
          drive.free_space > 0 ? Math.round(drive.free_space / (1024 * 1024 * 1024)) : 0;
        const usedPct =
          drive.total_space > 0
            ? Math.round(((drive.total_space - drive.free_space) / drive.total_space) * 100)
            : 0;
        return (
          <div key={drive.path} className="group relative">
            <button
              onClick={() => navigateToPath(drive.path)}
              className="hover:bg-xp-surface-light w-full rounded px-2 py-1 text-left text-xs transition-colors"
              aria-label={t('navigation.navigateTo', {
                name: drive.letter ? `${drive.letter}:` : drive.label,
              })}
            >
              <div className="flex items-center">
                <HardDrive
                  size={15}
                  className="text-xp-text-muted mr-2.5 flex-shrink-0"
                  aria-hidden="true"
                />
                <span className="text-xp-text flex-1 truncate">
                  {drive.letter ? `${drive.letter}:` : drive.label}
                </span>
                {totalGB > 0 && (
                  <span className="text-xp-text-muted ml-2 flex-shrink-0 pr-5">
                    {freeGB} GB free
                  </span>
                )}
              </div>
              {totalGB > 0 && (
                <div className="bg-xp-border ml-[25px] mt-1 h-1 overflow-hidden rounded-full">
                  <div
                    className={`h-full rounded-full transition-all ${usedPct > 90 ? 'bg-xp-red' : 'bg-xp-blue'}`}
                    style={{ width: `${usedPct}%` }}
                  />
                </div>
              )}
            </button>
            {/* Eject button — only shown for non-root/removable volumes */}
            {drive.path !== '/' && drive.path !== 'C:\\' && (
              <button
                className="text-xp-text-muted hover:text-xp-text hover:bg-xp-surface-light absolute right-1 top-1/2 -translate-y-1/2 rounded p-0.5 opacity-0 transition-opacity group-hover:opacity-100"
                onClick={(e) => handleEjectVolume(drive.path, e)}
                title={t('drives.eject')}
                aria-label={t('drives.eject')}
              >
                <ArrowUpFromLine size={12} />
              </button>
            )}
          </div>
        );
      })}
      {!isWindows && homePath && (
        <button
          onClick={() => navigateToPath(homePath)}
          className="hover:bg-xp-surface-light flex w-full items-center rounded px-2 py-1 text-xs transition-colors"
        >
          <User size={15} className="text-xp-cyan mr-2.5 flex-shrink-0" />{' '}
          {homePath.split('/').pop()}
        </button>
      )}
    </div>
  );
};

export default SidebarDrives;
