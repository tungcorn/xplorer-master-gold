import { useRef, useState, useEffect, useCallback, forwardRef } from 'react';
import { useWindowEvent } from '@/hooks/use-window-event';
import { isTauri } from '@/lib/transport';
import {
  Minus,
  Square,
  Copy,
  X,
  Plus,
  Columns,
  Rows,
  ChevronUp,
  ChevronRight,
  RefreshCw,
  FolderClosed,
  File,
  FileCode,
  GitCompareArrows,
  Cloud,
  HardDrive,
} from 'lucide-react';
import { ROOT_PATH } from '@/lib/constants';
import type { TabItem } from '@/types/split-view';
import { type FileCollection, getAllCollections, isQuickFilter } from '@/lib/collections';
import { renderIcon } from '@/lib/utils';
import { useTranslation } from 'react-i18next';

export interface TopBarHandle {
  // Retained for API compatibility — search now lives in the sidebar
}
interface TopBarProps {
  leftSidebarCollapsed: boolean;
  setLeftSidebarCollapsed: (collapsed: boolean) => void;
  currentPath: string;
  navigateToPath?: (path: string) => void;
  // Navigation
  navigateUp?: () => void;
  refetch?: () => void;
  navigateBackInHistory?: () => void;
  navigateForwardInHistory?: () => void;
  canNavigateBackInHistory?: () => boolean;
  canNavigateForwardInHistory?: () => boolean;
  // Tabs
  tabs?: TabItem[];
  activeTabId?: string;
  onSwitchTab?: (tabId: string) => void;
  onCloseTab?: (tabId: string) => void;
  // Split/tab actions
  onAddTab?: () => void;
  onSplitRight?: () => void;
  onSplitDown?: () => void;
  'data-tour'?: string;
  // Cross-tab selection
  crossTabTotalCount?: number;
  crossTabTabCount?: number;
  hasMultiTabSelection?: boolean;
  onOpenBatchActions?: () => void;
  onClearCrossTabSelection?: () => void;
  // Collection filters (collections applied as filters on current directory)
  activeCollectionFilter?: FileCollection | null;
  onToggleCollectionFilter?: (collection: FileCollection) => void;
  onClearCollectionFilter?: () => void;
}

const getTabIcon = (tab: TabItem) => {
  switch (tab.type) {
    case 'editor':
      return FileCode;
    case 'comparison':
      return GitCompareArrows;
    case 'gdrive':
    case 'gdrive-manager':
      return Cloud;
    case 'folder':
      return FolderClosed;
    default:
      return File;
  }
};

const TopBar = forwardRef<TopBarHandle, TopBarProps>(
  (
    {
      leftSidebarCollapsed,
      setLeftSidebarCollapsed,
      currentPath,
      navigateToPath,
      navigateUp,
      refetch,
      navigateBackInHistory,
      navigateForwardInHistory,
      canNavigateBackInHistory,
      canNavigateForwardInHistory,
      tabs,
      activeTabId,
      onSwitchTab,
      onCloseTab,
      onAddTab,
      onSplitRight,
      onSplitDown,
      'data-tour': dataTour,
      crossTabTotalCount = 0,
      crossTabTabCount = 0,
      hasMultiTabSelection = false,
      onOpenBatchActions,
      onClearCrossTabSelection,
      activeCollectionFilter,
      onToggleCollectionFilter,
      onClearCollectionFilter,
    },
    _ref,
  ) => {
    const { t } = useTranslation();
    const [isMaximized, setIsMaximized] = useState(false);
    const [filterDropdownOpen, setFilterDropdownOpen] = useState(false);
    const [quickFilters, setQuickFilters] = useState<FileCollection[]>(() =>
      getAllCollections().filter(isQuickFilter),
    );
    const filterDropdownRef = useRef<HTMLDivElement>(null);

    // Load quick-filter collections and sync
    useWindowEvent('collections-changed', () =>
      setQuickFilters(getAllCollections().filter(isQuickFilter)),
    );

    // Close dropdown on outside click
    useEffect(() => {
      if (!filterDropdownOpen) return;
      const close = (e: MouseEvent) => {
        if (filterDropdownRef.current && !filterDropdownRef.current.contains(e.target as Node)) {
          setFilterDropdownOpen(false);
        }
      };
      document.addEventListener('mousedown', close);
      return () => document.removeEventListener('mousedown', close);
    }, [filterDropdownOpen]);
    const appWindowRef = useRef<Awaited<
      ReturnType<typeof import('@tauri-apps/api/window').getCurrentWindow>
    > | null>(null);
    const isMac = navigator.platform.toUpperCase().includes('MAC');

    useEffect(() => {
      if (!isTauri()) return;
      let debounceTimer: ReturnType<typeof setTimeout> | null = null;
      let cancelled = false;
      const cleanupRef = { current: null as (() => void) | null };
      (async () => {
        const { getCurrentWindow } = await import('@tauri-apps/api/window');
        const win = getCurrentWindow();
        if (cancelled) return;
        appWindowRef.current = win;
        win
          .isMaximized()
          .then(setIsMaximized)
          .catch((err: unknown) => console.warn('Failed to check maximized state:', err));
        const unlisten = win.onResized(() => {
          if (debounceTimer) clearTimeout(debounceTimer);
          debounceTimer = setTimeout(() => {
            win
              .isMaximized()
              .then(setIsMaximized)
              .catch((err: unknown) => console.warn('Failed to check maximized state:', err));
          }, 150);
        });
        // Store unlisten for cleanup
        if (!cancelled) {
          cleanupRef.current = async () => {
            if (debounceTimer) clearTimeout(debounceTimer);
            (await unlisten)();
          };
        }
      })();
      return () => {
        cancelled = true;
        if (debounceTimer) clearTimeout(debounceTimer);
        cleanupRef.current?.();
      };
    }, []);

    const breadcrumbSegments = useCallback(() => {
      if (!currentPath || currentPath.startsWith('xplorer://')) return [];
      const normalized = currentPath.replace(/\\/g, '/');
      const parts = normalized.split('/').filter(Boolean);
      const segs: { name: string; fullPath: string }[] = [];
      let acc = '';
      for (const part of parts) {
        acc = acc ? `${acc}/${part}` : part;
        const fullPath = acc.includes(':') && !acc.includes(':/') ? `${acc}/` : acc;
        segs.push({ name: part, fullPath });
      }
      return segs;
    }, [currentPath]);

    return (
      <div data-tour={dataTour} className="flex-none">
        {/* ── Row 1: Tab strip + window controls ──────────────── */}
        <div
          className="flex items-center"
          style={{ height: 40, background: 'var(--xp-titlebar)' }}
          onMouseDown={(e) => {
            if (
              !(e.target as HTMLElement).closest(
                'button, input, a, select, textarea, [role="button"], [role="tab"]',
              )
            ) {
              e.preventDefault();
              appWindowRef.current?.startDragging();
            }
          }}
          onDoubleClick={(e) => {
            if (
              !(e.target as HTMLElement).closest(
                'button, input, a, select, textarea, [role="button"], [role="tab"]',
              )
            ) {
              appWindowRef.current?.toggleMaximize();
            }
          }}
        >
          {/* Mac traffic lights offset */}
          {isMac && <div style={{ width: 72, flexShrink: 0 }} />}

          {/* Tabs */}
          <div
            className="scrollbar-none flex min-w-0 flex-1 items-end overflow-x-auto px-1"
            style={{ paddingTop: 6 }}
          >
            {tabs?.map((tab) => {
              const TabIcon = getTabIcon(tab);
              const isActive = activeTabId === tab.id;
              return (
                <div
                  key={tab.id}
                  role="tab"
                  aria-selected={isActive}
                  className={`group flex min-w-0 max-w-[240px] flex-shrink-0 cursor-pointer items-center gap-1.5 px-3 ${
                    isActive ? 'text-xp-text' : 'text-xp-text-muted hover:text-xp-text-secondary'
                  }`}
                  style={{
                    height: 34,
                    fontSize: 13,
                    borderRadius: '10px 10px 0 0',
                    background: isActive ? 'var(--xp-bg)' : 'transparent',
                  }}
                  onClick={() => onSwitchTab?.(tab.id)}
                >
                  <TabIcon size={13} className="flex-shrink-0 opacity-70" />
                  <span className="truncate font-medium">{tab.name}</span>
                  {tabs && tabs.length > 1 && (
                    <button
                      onClick={(e) => {
                        e.stopPropagation();
                        onCloseTab?.(tab.id);
                      }}
                      className="ml-0.5 flex-shrink-0 rounded p-0.5 opacity-0 transition-opacity hover:bg-white/10 group-hover:opacity-100"
                      style={{ fontSize: 14 }}
                      aria-label={`Close ${tab.name}`}
                    >
                      <X size={12} />
                    </button>
                  )}
                </div>
              );
            })}
            {/* New tab */}
            {onAddTab && (
              <button
                onClick={onAddTab}
                className="text-xp-text-muted hover:text-xp-text flex-shrink-0 rounded p-1 transition-colors hover:bg-white/[0.06]"
                style={{
                  width: 28,
                  height: 28,
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'center',
                }}
                title={t('topBar.newTabShortcut')}
                aria-label={t('topBar.newTab')}
              >
                <Plus size={14} />
              </button>
            )}
          </div>

          {/* Split actions */}
          <div className="flex flex-shrink-0 items-center gap-0.5 px-1">
            {onSplitRight && (
              <button
                onClick={onSplitRight}
                className="text-xp-text-muted hover:text-xp-text rounded p-1 hover:bg-white/[0.06]"
                title={t('topBar.splitRightShortcut')}
                aria-label={t('topBar.splitRight')}
              >
                <Columns size={14} />
              </button>
            )}
            {onSplitDown && (
              <button
                onClick={onSplitDown}
                className="text-xp-text-muted hover:text-xp-text rounded p-1 hover:bg-white/[0.06]"
                title={t('topBar.splitDownShortcut')}
                aria-label={t('topBar.splitDown')}
              >
                <Rows size={14} />
              </button>
            )}
          </div>

          {/* Window controls (Windows/Linux) */}
          {!isMac && (
            <div
              className="flex flex-shrink-0 items-center"
              role="toolbar"
              aria-label="Window controls"
            >
              <button
                onClick={() => appWindowRef.current?.minimize()}
                className="rounded p-1.5 transition-colors hover:bg-white/[0.06]"
                aria-label={t('topBar.minimize')}
              >
                <Minus size={14} />
              </button>
              <button
                onClick={() => appWindowRef.current?.toggleMaximize()}
                className="rounded p-1.5 transition-colors hover:bg-white/[0.06]"
                aria-label={isMaximized ? t('topBar.restore') : t('topBar.maximize')}
              >
                {isMaximized ? <Copy size={14} /> : <Square size={14} />}
              </button>
              <button
                onClick={() => appWindowRef.current?.close()}
                className="xp-close-btn rounded p-1.5 transition-colors"
                aria-label={t('topBar.closeWindow')}
              >
                <X size={14} />
              </button>
            </div>
          )}
        </div>

        {/* ── Row 2: Toolbar (hamburger + nav + breadcrumb + filter) ── */}
        <div
          className="border-xp-border flex items-center gap-0.5 border-b px-2"
          style={{ height: 42, background: 'var(--xp-bg)' }}
        >
          {/* Sidebar toggle */}
          <button
            onClick={() => setLeftSidebarCollapsed(!leftSidebarCollapsed)}
            className="flex-shrink-0 rounded p-1 transition-colors hover:bg-white/[0.06]"
            aria-label={t('topBar.toggleSidebar')}
            title={t('topBar.toggleSidebarShortcut')}
          >
            <svg className="h-4 w-4" fill="currentColor" viewBox="0 0 20 20">
              <path
                fillRule="evenodd"
                d="M3 5a1 1 0 011-1h12a1 1 0 110 2H4a1 1 0 01-1-1zM3 10a1 1 0 011-1h12a1 1 0 110 2H4a1 1 0 01-1-1zM3 15a1 1 0 011-1h12a1 1 0 110 2H4a1 1 0 01-1-1z"
                clipRule="evenodd"
              />
            </svg>
          </button>

          <div
            className="mx-0.5 h-5 w-px flex-shrink-0"
            style={{ background: 'rgba(255,255,255,0.08)' }}
          />

          {/* Nav buttons */}
          {navigateBackInHistory && (
            <button
              onClick={navigateBackInHistory}
              disabled={!canNavigateBackInHistory?.()}
              className="flex-shrink-0 rounded p-1 transition-colors hover:bg-white/[0.06] disabled:opacity-30"
              title={t('topBar.goBack')}
              aria-label={t('topBar.goBack')}
            >
              <svg className="h-4 w-4" fill="currentColor" viewBox="0 0 20 20">
                <path
                  fillRule="evenodd"
                  d="M12.707 5.293a1 1 0 010 1.414L9.414 10l3.293 3.293a1 1 0 01-1.414 1.414l-4-4a1 1 0 010-1.414l4-4a1 1 0 011.414 0z"
                  clipRule="evenodd"
                />
              </svg>
            </button>
          )}
          {navigateForwardInHistory && (
            <button
              onClick={navigateForwardInHistory}
              disabled={!canNavigateForwardInHistory?.()}
              className="flex-shrink-0 rounded p-1 transition-colors hover:bg-white/[0.06] disabled:opacity-30"
              title={t('topBar.goForward')}
              aria-label={t('topBar.goForward')}
            >
              <svg className="h-4 w-4" fill="currentColor" viewBox="0 0 20 20">
                <path
                  fillRule="evenodd"
                  d="M7.293 14.707a1 1 0 010-1.414L10.586 10 7.293 6.707a1 1 0 011.414-1.414l4 4a1 1 0 010 1.414l-4 4a1 1 0 01-1.414 0z"
                  clipRule="evenodd"
                />
              </svg>
            </button>
          )}
          {navigateUp && (
            <button
              onClick={navigateUp}
              disabled={currentPath === ROOT_PATH}
              className="flex-shrink-0 rounded p-1 transition-colors hover:bg-white/[0.06] disabled:opacity-30"
              title={t('topBar.goUp')}
              aria-label={t('topBar.goUp')}
            >
              <ChevronUp size={16} />
            </button>
          )}
          {refetch && (
            <button
              onClick={refetch}
              className="flex-shrink-0 rounded p-1 transition-colors hover:bg-white/[0.06]"
              title={t('topBar.refresh')}
              aria-label={t('topBar.refresh')}
            >
              <RefreshCw size={14} />
            </button>
          )}

          <div
            className="mx-1 h-5 w-px flex-shrink-0"
            style={{ background: 'rgba(255,255,255,0.08)' }}
          />

          {/* Inline breadcrumb */}
          <nav
            aria-label="Breadcrumb"
            className="scrollbar-none flex min-w-0 flex-1 items-center gap-0.5 overflow-x-auto"
          >
            {(() => {
              const segs = breadcrumbSegments();
              if (segs.length === 0) {
                return <span className="text-xp-text-muted text-[13px]">{currentPath}</span>;
              }
              return segs.map((seg, i) => (
                <span key={seg.fullPath} className="flex flex-shrink-0 items-center">
                  {i > 0 && (
                    <ChevronRight size={12} className="text-xp-text-muted mx-0.5 opacity-60" />
                  )}
                  <button
                    onClick={() => navigateToPath?.(seg.fullPath)}
                    className={`max-w-[160px] truncate rounded px-1.5 py-0.5 text-[13px] transition-colors hover:bg-white/[0.03] ${
                      i === segs.length - 1
                        ? 'text-xp-text font-medium'
                        : 'text-xp-text-muted hover:text-xp-text'
                    }`}
                    title={seg.fullPath}
                  >
                    {i === 0 && /^[A-Za-z]:$/.test(seg.name) ? (
                      <span className="flex items-center gap-1">
                        <HardDrive size={12} className="flex-shrink-0" />
                        {seg.name}
                      </span>
                    ) : (
                      seg.name
                    )}
                  </button>
                </span>
              ));
            })()}
          </nav>

          {/* Quick Filter dropdown */}
          <div ref={filterDropdownRef} style={{ position: 'relative' }} className="flex-shrink-0">
            <button
              onClick={() => setFilterDropdownOpen(!filterDropdownOpen)}
              className={`flex flex-shrink-0 items-center gap-1 rounded p-1 transition-colors ${
                activeCollectionFilter
                  ? 'text-xp-text'
                  : 'text-xp-text-muted hover:text-xp-text hover:bg-white/[0.06]'
              }`}
              style={
                activeCollectionFilter
                  ? {
                      backgroundColor: `${activeCollectionFilter.color}20`,
                      border: `1px solid ${activeCollectionFilter.color}40`,
                    }
                  : undefined
              }
              title={t('topBar.quickFilters')}
              aria-label={t('topBar.quickFilters')}
              aria-expanded={filterDropdownOpen}
            >
              <svg
                width="14"
                height="14"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                strokeWidth="2"
                strokeLinecap="round"
                strokeLinejoin="round"
              >
                <polygon points="22 3 2 3 10 12.46 10 19 14 21 14 12.46 22 3" />
              </svg>
              {activeCollectionFilter && (
                <span
                  style={{
                    fontSize: '11px',
                    fontWeight: 500,
                    maxWidth: '80px',
                    overflow: 'hidden',
                    textOverflow: 'ellipsis',
                    whiteSpace: 'nowrap',
                  }}
                >
                  {activeCollectionFilter.name}
                </span>
              )}
            </button>
            {filterDropdownOpen && (
              <div
                style={{
                  position: 'absolute',
                  top: '100%',
                  left: 0,
                  zIndex: 9999,
                  minWidth: '200px',
                  maxHeight: '320px',
                  overflowY: 'auto',
                  borderRadius: '16px',
                  backgroundColor: 'var(--xp-popover)',
                  border: '1px solid rgba(255,255,255,0.08)',
                  boxShadow: '0 12px 32px rgba(0,0,0,0.35)',
                  padding: '4px',
                  marginTop: '4px',
                }}
              >
                {quickFilters.map((col) => {
                  const isActive = activeCollectionFilter?.id === col.id;
                  return (
                    <button
                      key={col.id}
                      className="flex w-full items-center rounded px-3 py-1.5 text-xs transition-colors"
                      style={{
                        color: 'var(--xp-text)',
                        backgroundColor: isActive ? `${col.color}15` : 'transparent',
                        borderLeft: isActive ? `3px solid ${col.color}` : '3px solid transparent',
                      }}
                      onMouseEnter={(e) => {
                        if (!isActive) {
                          (e.currentTarget as HTMLElement).style.backgroundColor =
                            'rgba(255,255,255,0.03)';
                        }
                      }}
                      onMouseLeave={(e) => {
                        if (!isActive) {
                          (e.currentTarget as HTMLElement).style.backgroundColor = 'transparent';
                        }
                      }}
                      onClick={() => {
                        onToggleCollectionFilter?.(col);
                        setFilterDropdownOpen(false);
                      }}
                    >
                      <span
                        style={{
                          marginRight: '8px',
                          fontSize: '14px',
                          display: 'inline-flex',
                          alignItems: 'center',
                        }}
                      >
                        {renderIcon(col.icon, 14)}
                      </span>
                      <span style={{ flex: 1, textAlign: 'left' }}>{col.name}</span>
                      {isActive && (
                        <span
                          style={{
                            width: '6px',
                            height: '6px',
                            borderRadius: '50%',
                            backgroundColor: col.color,
                            marginLeft: '8px',
                            flexShrink: 0,
                          }}
                        />
                      )}
                    </button>
                  );
                })}
                {activeCollectionFilter && (
                  <>
                    <div
                      style={{
                        height: '1px',
                        backgroundColor: 'rgba(255,255,255,0.08)',
                        margin: '4px 0',
                      }}
                    />
                    <button
                      className="flex w-full items-center rounded px-3 py-1.5 text-xs transition-colors"
                      style={{ color: 'var(--xp-text-muted)' }}
                      onMouseEnter={(e) => {
                        (e.currentTarget as HTMLElement).style.backgroundColor =
                          'rgba(255,255,255,0.03)';
                      }}
                      onMouseLeave={(e) => {
                        (e.currentTarget as HTMLElement).style.backgroundColor = 'transparent';
                      }}
                      onClick={() => {
                        onClearCollectionFilter?.();
                        setFilterDropdownOpen(false);
                      }}
                    >
                      <X size={12} style={{ marginRight: '8px' }} />
                      {t('topBar.clearFilter')}
                    </button>
                  </>
                )}
              </div>
            )}
          </div>
        </div>

        {/* ── Cross-tab selection banner ──────────────────────── */}
        {hasMultiTabSelection && crossTabTotalCount > 0 && (
          <div
            style={{
              display: 'flex',
              alignItems: 'center',
              gap: 8,
              padding: '4px 12px',
              background: 'color-mix(in srgb, var(--xp-blue) 12%, var(--xp-surface))',
              borderTop: '1px solid color-mix(in srgb, var(--xp-blue) 25%, var(--xp-border))',
              fontSize: 12,
              color: 'var(--xp-text)',
            }}
          >
            {/* Selection badge */}
            <span
              style={{
                display: 'inline-flex',
                alignItems: 'center',
                gap: 4,
                padding: '2px 8px',
                borderRadius: 10,
                background: 'var(--xp-blue)',
                color: '#fff',
                fontSize: 11,
                fontWeight: 600,
              }}
            >
              <svg
                width="12"
                height="12"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                strokeWidth="2.5"
                strokeLinecap="round"
                strokeLinejoin="round"
              >
                <polyline points="9 11 12 14 22 4" />
                <path d="M21 12v7a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11" />
              </svg>
              {crossTabTotalCount !== 1 || crossTabTabCount !== 1
                ? t('topBar.crossTabSelectionPlural', {
                    fileCount: crossTabTotalCount,
                    tabCount: crossTabTabCount,
                  })
                : t('topBar.crossTabSelection', {
                    fileCount: crossTabTotalCount,
                    tabCount: crossTabTabCount,
                  })}
            </span>

            <span style={{ color: 'var(--xp-text-muted)', fontSize: 11 }}>
              {t('topBar.crossTabHint')}
            </span>

            {/* Spacer */}
            <div style={{ flex: 1 }} />

            {/* Batch Actions button */}
            {onOpenBatchActions && (
              <button
                onClick={onOpenBatchActions}
                style={{
                  padding: '3px 10px',
                  fontSize: 11,
                  fontWeight: 500,
                  borderRadius: 5,
                  border: '1px solid var(--xp-blue)',
                  background: 'var(--xp-blue)',
                  color: '#fff',
                  cursor: 'pointer',
                  display: 'flex',
                  alignItems: 'center',
                  gap: 4,
                  transition: 'opacity 0.15s ease',
                }}
                onMouseEnter={(e) => {
                  (e.currentTarget as HTMLElement).style.opacity = '0.85';
                }}
                onMouseLeave={(e) => {
                  (e.currentTarget as HTMLElement).style.opacity = '1';
                }}
                title={t('topBar.batchActionsDesc')}
              >
                <svg
                  width="12"
                  height="12"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  strokeWidth="2"
                  strokeLinecap="round"
                  strokeLinejoin="round"
                >
                  <rect x="3" y="3" width="18" height="18" rx="2" />
                  <line x1="3" y1="9" x2="21" y2="9" />
                  <line x1="9" y1="21" x2="9" y2="9" />
                </svg>
                {t('topBar.batchActions')}
              </button>
            )}

            {/* Clear Selection button */}
            {onClearCrossTabSelection && (
              <button
                onClick={onClearCrossTabSelection}
                style={{
                  padding: '3px 10px',
                  fontSize: 11,
                  fontWeight: 500,
                  borderRadius: 5,
                  border: '1px solid var(--xp-border)',
                  background: 'var(--xp-surface-light)',
                  color: 'var(--xp-text-muted)',
                  cursor: 'pointer',
                  display: 'flex',
                  alignItems: 'center',
                  gap: 4,
                  transition: 'background 0.15s ease',
                }}
                onMouseEnter={(e) => {
                  (e.currentTarget as HTMLElement).style.background = 'var(--xp-surface)';
                  (e.currentTarget as HTMLElement).style.color = 'var(--xp-text)';
                }}
                onMouseLeave={(e) => {
                  (e.currentTarget as HTMLElement).style.background = 'var(--xp-surface-light)';
                  (e.currentTarget as HTMLElement).style.color = 'var(--xp-text-muted)';
                }}
                title={t('topBar.clearSelectionDesc')}
              >
                <X size={12} />
                {t('topBar.clearSelection')}
              </button>
            )}
          </div>
        )}
      </div>
    );
  },
);
TopBar.displayName = 'TopBar';
export default TopBar;
