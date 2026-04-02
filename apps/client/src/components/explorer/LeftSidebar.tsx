import React, { useRef, useImperativeHandle, forwardRef } from 'react';
import { FileEntry } from '@/lib/tauri-api';
import SearchResultsPanel, {
  type SearchResultsPanelHandle,
} from '@/components/explorer/SearchResultsPanel';
import SidebarQuickAccess from '@/components/explorer/sidebar/SidebarQuickAccess';
import SidebarBookmarks from '@/components/explorer/sidebar/SidebarBookmarks';
import SidebarDrives from '@/components/explorer/sidebar/SidebarDrives';
import SidebarFileTree from '@/components/explorer/sidebar/SidebarFileTree';

export interface LeftSidebarHandle {
  focusSearch: () => void;
}

interface LeftSidebarProps {
  currentPath: string;
  navigateToPath: (path: string) => void;
  handleFileClick: (file: FileEntry) => void;
  handleFileRightClick?: (file: FileEntry, event: React.MouseEvent) => void;
  handleFileOpen?: (file: FileEntry) => void;
  getFileIcon: (file: FileEntry) => React.ReactNode;
  width?: number;
  searchPanelOpen?: boolean;
  onToggleSearchPanel?: () => void;
  'data-tour'?: string;
}

const LeftSidebar = forwardRef<LeftSidebarHandle, LeftSidebarProps>(
  (
    {
      currentPath,
      navigateToPath,
      handleFileClick,
      handleFileRightClick,
      handleFileOpen,
      getFileIcon,
      width,
      searchPanelOpen = false,
      onToggleSearchPanel,
      'data-tour': dataTour,
    },
    ref,
  ) => {
    const searchPanelRef = useRef<SearchResultsPanelHandle>(null);

    useImperativeHandle(ref, () => ({
      focusSearch: () => {
        if (!searchPanelOpen && onToggleSearchPanel) {
          onToggleSearchPanel();
          setTimeout(() => searchPanelRef.current?.focus(), 100);
        } else {
          searchPanelRef.current?.focus();
        }
      },
    }));

    // When search panel is open, show it instead of the explorer content
    if (searchPanelOpen) {
      return (
        <nav
          data-tour={dataTour}
          role="navigation"
          aria-label="File explorer sidebar"
          className="bg-xp-surface border-xp-border flex flex-shrink-0 flex-col border-r"
          style={{ width: width ?? 200, minHeight: 0, overflow: 'hidden' }}
        >
          <SearchResultsPanel
            ref={searchPanelRef}
            basePath={currentPath}
            navigateToPath={navigateToPath}
            onFileSelect={handleFileClick}
            onFileOpen={handleFileOpen}
          />
        </nav>
      );
    }

    return (
      <nav
        data-tour={dataTour}
        role="navigation"
        aria-label="File explorer sidebar"
        className="bg-xp-surface border-xp-border flex flex-shrink-0 flex-col border-r"
        style={{ width: width ?? 200, minHeight: 0, overflow: 'hidden' }}
      >
        <SidebarQuickAccess currentPath={currentPath} navigateToPath={navigateToPath} />
        <div className="bg-xp-border mx-3 h-px" />
        <SidebarDrives navigateToPath={navigateToPath} />
        <div className="bg-xp-border mx-3 h-px" />
        <SidebarBookmarks
          currentPath={currentPath}
          navigateToPath={navigateToPath}
          handleFileRightClick={handleFileRightClick}
        />
        <div className="bg-xp-border mx-3 h-px" />
        <SidebarFileTree
          currentPath={currentPath}
          navigateToPath={navigateToPath}
          handleFileClick={handleFileClick}
          handleFileRightClick={handleFileRightClick}
          getFileIcon={getFileIcon}
        />
      </nav>
    );
  },
);

LeftSidebar.displayName = 'LeftSidebar';

export default LeftSidebar;
