import { useState, useEffect } from 'react';
import { Home, FileText, Download, Monitor, Image, Cloud } from 'lucide-react';
import { TauriAPI } from '@/lib/tauri-api';
import { PATH_SEPARATOR, isWindows, isMac } from '@/lib/constants';
import { useTranslation } from 'react-i18next';

interface UserDirectories {
  home: string;
  documents: string;
  downloads: string;
  desktop: string;
  pictures: string;
  videos: string;
  music: string;
}

interface SidebarQuickAccessProps {
  currentPath: string;
  navigateToPath: (path: string) => void;
  filterText?: string;
}

const SidebarQuickAccess = ({
  currentPath,
  navigateToPath,
  filterText = '',
}: SidebarQuickAccessProps) => {
  const { t } = useTranslation();
  const [userDirectories, setUserDirectories] = useState<UserDirectories | null>(null);
  const [iCloudPath, setICloudPath] = useState<string | null>(null);

  useEffect(() => {
    const load = async () => {
      try {
        const userDirs = await TauriAPI.getUserDirectories();
        setUserDirectories(userDirs);
        if (isMac) {
          const cloudDocsPath = `${userDirs.home}/Library/Mobile Documents/com~apple~CloudDocs`;
          const exists = await TauriAPI.fileExists(cloudDocsPath);
          setICloudPath(exists ? cloudDocsPath : null);
        }
      } catch (error) {
        console.error('Failed to load user directories:', error);
        const home = isWindows ? 'C:\\Users\\Public' : '/home/user';
        setUserDirectories({
          home,
          documents: `${home + PATH_SEPARATOR}Documents`,
          downloads: `${home + PATH_SEPARATOR}Downloads`,
          desktop: `${home + PATH_SEPARATOR}Desktop`,
          pictures: `${home + PATH_SEPARATOR}Pictures`,
          videos: `${home + PATH_SEPARATOR}Videos`,
          music: `${home + PATH_SEPARATOR}Music`,
        });
      }
    };
    load();
  }, []);

  return (
    <div role="region" aria-label="Quick access" className="space-y-0.5 px-3 py-2">
      {userDirectories &&
        (
          [
            {
              path: userDirectories.home,
              Icon: Home,
              color: 'text-xp-blue',
              labelKey: 'sidebar.home' as const,
            },
            {
              path: userDirectories.documents,
              Icon: FileText,
              color: 'text-xp-orange',
              labelKey: 'sidebar.documents' as const,
            },
            {
              path: userDirectories.downloads,
              Icon: Download,
              color: 'text-xp-green',
              labelKey: 'sidebar.downloads' as const,
            },
            {
              path: userDirectories.desktop,
              Icon: Monitor,
              color: 'text-xp-cyan',
              labelKey: 'sidebar.desktop' as const,
            },
            {
              path: userDirectories.pictures,
              Icon: Image,
              color: 'text-xp-orange',
              labelKey: 'sidebar.pictures' as const,
            },
          ] as const
        )
          .filter(
            ({ labelKey }) =>
              !filterText || t(labelKey).toLowerCase().includes(filterText.toLowerCase()),
          )
          .map(({ path, Icon, color, labelKey }) => {
            const label = t(labelKey);
            const isActive = currentPath === path;
            return (
              <button
                key={labelKey}
                onClick={() => navigateToPath(path)}
                className={`flex w-full items-center rounded px-2 py-1 text-xs transition-colors ${
                  isActive ? 'text-xp-text bg-white/[0.04]' : 'text-xp-text hover:bg-white/[0.03]'
                }`}
                aria-label={t('sidebar.navigateTo', { label })}
              >
                <Icon
                  size={15}
                  className={`mr-2.5 flex-shrink-0 ${isActive ? 'text-xp-blue' : color}`}
                  aria-hidden="true"
                />
                {label}
              </button>
            );
          })}
      {iCloudPath &&
        (() => {
          const isActive = currentPath === iCloudPath;
          const label = t('sidebar.icloudDrive');
          return (
            <button
              key="icloud"
              onClick={() => navigateToPath(iCloudPath)}
              className={`flex w-full items-center rounded px-2 py-1 text-xs transition-colors ${
                isActive ? 'text-xp-text bg-white/[0.04]' : 'text-xp-text hover:bg-white/[0.03]'
              }`}
              aria-label={t('sidebar.navigateTo', { label })}
            >
              <Cloud
                size={15}
                className={`mr-2.5 flex-shrink-0 ${isActive ? 'text-xp-blue' : 'text-xp-cyan'}`}
                aria-hidden="true"
              />
              {label}
            </button>
          );
        })()}
    </div>
  );
};

export default SidebarQuickAccess;
