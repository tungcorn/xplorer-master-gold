import React, { useRef, useState, useImperativeHandle, forwardRef } from 'react';
import { FileEntry } from '@/lib/tauri-api';
import { Search } from 'lucide-react';
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
    const [filterText, setFilterText] = useState('');

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
          style={{ width: width ?? 232, minHeight: 0, overflow: 'hidden' }}
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
        style={{ width: width ?? 232, minHeight: 0, overflow: 'hidden' }}
      >
        <div className="flex-shrink-0 px-2 pb-1 pt-2">
          <div
            className="flex items-center rounded-lg px-2"
            style={{ height: 32, background: '#25282b' }}
          >
            <Search size={14} className="text-xp-text-muted mr-2 flex-shrink-0" />
            <input
              type="text"
              value={filterText}
              onChange={(e) => setFilterText(e.target.value)}
              className="text-xp-text placeholder:text-xp-text-muted w-full bg-transparent text-[13px] outline-none"
              placeholder="Filter..."
            />
          </div>
        </div>
        <SidebarQuickAccess
          currentPath={currentPath}
          navigateToPath={navigateToPath}
          filterText={filterText}
        />
        <div style={{ height: 12 }} />
        <SidebarDrives navigateToPath={navigateToPath} />
        <div style={{ height: 12 }} />
        <SidebarBookmarks
          currentPath={currentPath}
          navigateToPath={navigateToPath}
          handleFileRightClick={handleFileRightClick}
          filterText={filterText}
        />
        <div style={{ height: 12 }} />
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
