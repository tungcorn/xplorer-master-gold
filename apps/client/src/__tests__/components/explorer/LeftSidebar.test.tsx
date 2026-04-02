import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import '@testing-library/jest-dom';
import LeftSidebar from '@/components/explorer/LeftSidebar';
import { FileEntry, TauriAPI } from '@/lib/tauri-api';

// Mock constants
vi.mock('@/lib/constants', () => ({
  isWindows: true,
  PATH_SEPARATOR: '\\',
  ROOT_PATH: 'C:\\',
}));

// Mock collections
vi.mock('@/lib/collections', () => ({
  getCollections: vi.fn(() => []),
  getAllCollections: vi.fn(() => []),
  deleteCollection: vi.fn(),
  isQuickFilter: vi.fn(() => false),
  isSmartFolder: vi.fn(() => false),
}));

// Mock folder-colors
vi.mock('@/lib/folder-colors', () => ({
  getFolderColorHex: vi.fn(() => null),
  getAllFolderColors: vi.fn(() => ({})),
}));

// Mock path-bookmarks
vi.mock('@/lib/path-bookmarks', () => ({
  getPathBookmarks: vi.fn(() => []),
  removePathBookmark: vi.fn(),
  getFolderName: vi.fn((path: string) => path.split(/[\\/]/).pop() || path),
}));

// Mock extension-host
vi.mock('@/lib/extension-host', () => ({
  extensionHost: {
    isExtensionScheme: vi.fn(() => false),
    getTabRenderer: vi.fn(() => null),
    getNavigationEntries: vi.fn(() => []),
    getSidebarTabs: vi.fn(() => []),
    getSidebarTabRenderer: vi.fn(() => null),
    onChange: vi.fn(() => () => {}),
  },
}));

// Extend TauriAPI mock for LeftSidebar-specific methods
vi.mock('@/lib/tauri-api', () => ({
  TauriAPI: {
    readDirectory: vi.fn(() => Promise.resolve([])),
    getUserDirectories: vi.fn(() =>
      Promise.resolve({
        home: 'C:\\Users\\Test',
        documents: 'C:\\Users\\Test\\Documents',
        downloads: 'C:\\Users\\Test\\Downloads',
        desktop: 'C:\\Users\\Test\\Desktop',
        pictures: 'C:\\Users\\Test\\Pictures',
        videos: 'C:\\Users\\Test\\Videos',
        music: 'C:\\Users\\Test\\Music',
      }),
    ),
    listDrives: vi.fn(() =>
      Promise.resolve([
        {
          letter: 'C',
          label: 'Local Disk',
          path: 'C:\\',
          total_space: 500000000000,
          free_space: 200000000000,
        },
        {
          letter: 'D',
          label: 'Data',
          path: 'D:\\',
          total_space: 1000000000000,
          free_space: 500000000000,
        },
      ]),
    ),
    getBookmarks: vi.fn(() => Promise.resolve([])),
    removeBookmark: vi.fn(() => Promise.resolve()),
    getRecentFiles: vi.fn(() => Promise.resolve([])),
    getFileIcon: vi.fn(() => '📄'),
    formatFileSize: vi.fn(() => '1 KB'),
    formatDate: vi.fn(() => '2024-01-01'),
  },
  FileEntry: {},
}));

describe('LeftSidebar', () => {
  const mockFiles: FileEntry[] = [
    {
      name: 'folder1',
      path: 'C:\\Users\\Test\\folder1',
      size: 0,
      is_dir: true,
      modified: Date.now(),
      file_type: 'folder',
    },
    {
      name: 'file1.txt',
      path: 'C:\\Users\\Test\\file1.txt',
      size: 1024,
      is_dir: false,
      modified: Date.now(),
      file_type: 'text',
    },
  ];

  const mockProps = {
    currentPath: 'C:\\Users\\Test',
    files: mockFiles,
    navigateToPath: vi.fn(),
    handleFileClick: vi.fn(),
    handleFileRightClick: vi.fn(),
    getFileIcon: vi.fn((file: FileEntry) => (file.is_dir ? '📁' : '📄')),
    sortBy: 'name',
    sortOrder: 'asc' as const,
  };

  beforeEach(() => {
    vi.clearAllMocks();
    localStorage.clear();
  });

  describe('Quick Access Section', () => {
    it('renders Home quick access item', async () => {
      render(<LeftSidebar {...mockProps} />);

      await waitFor(() => {
        expect(screen.getByText('home')).toBeInTheDocument();
      });
    });

    it('renders Documents quick access item', async () => {
      render(<LeftSidebar {...mockProps} />);

      await waitFor(() => {
        expect(screen.getByText('documents')).toBeInTheDocument();
      });
    });

    it('renders Downloads quick access item', async () => {
      render(<LeftSidebar {...mockProps} />);

      await waitFor(() => {
        expect(screen.getByText('downloads')).toBeInTheDocument();
      });
    });

    it('renders Desktop quick access item', async () => {
      render(<LeftSidebar {...mockProps} />);

      await waitFor(() => {
        expect(screen.getByText('desktop')).toBeInTheDocument();
      });
    });

    it('renders Pictures quick access item', async () => {
      render(<LeftSidebar {...mockProps} />);

      await waitFor(() => {
        expect(screen.getByText('pictures')).toBeInTheDocument();
      });
    });

    it('navigates to Home when Home is clicked', async () => {
      render(<LeftSidebar {...mockProps} />);

      await waitFor(() => {
        expect(screen.getByText('home')).toBeInTheDocument();
      });

      fireEvent.click(screen.getByText('home'));
      expect(mockProps.navigateToPath).toHaveBeenCalledWith('C:\\Users\\Test');
    });

    it('navigates to Documents when Documents is clicked', async () => {
      render(<LeftSidebar {...mockProps} />);

      await waitFor(() => {
        expect(screen.getByText('documents')).toBeInTheDocument();
      });

      fireEvent.click(screen.getByText('documents'));
      expect(mockProps.navigateToPath).toHaveBeenCalledWith('C:\\Users\\Test\\Documents');
    });

    it('navigates to Downloads when Downloads is clicked', async () => {
      render(<LeftSidebar {...mockProps} />);

      await waitFor(() => {
        expect(screen.getByText('downloads')).toBeInTheDocument();
      });

      fireEvent.click(screen.getByText('downloads'));
      expect(mockProps.navigateToPath).toHaveBeenCalledWith('C:\\Users\\Test\\Downloads');
    });
  });

  describe('Bookmarks Section', () => {
    it('shows empty bookmarks message when none exist', async () => {
      render(<LeftSidebar {...mockProps} />);

      await waitFor(() => {
        expect(screen.getByText('noBookmarks')).toBeInTheDocument();
      });
    });

    it('renders bookmark items when bookmarks exist', async () => {
      vi.mocked(TauriAPI.getBookmarks).mockResolvedValueOnce([
        { name: 'MyFolder', path: 'C:\\Users\\Test\\MyFolder', is_dir: true },
        { name: 'readme.md', path: 'C:\\Users\\Test\\readme.md', is_dir: false },
      ]);

      render(<LeftSidebar {...mockProps} />);

      await waitFor(() => {
        expect(screen.getByText('MyFolder')).toBeInTheDocument();
        expect(screen.getByText('readme.md')).toBeInTheDocument();
      });
    });

    it('navigates to bookmark when clicked', async () => {
      vi.mocked(TauriAPI.getBookmarks).mockResolvedValueOnce([
        { name: 'MyFolder', path: 'C:\\Users\\Test\\MyFolder', is_dir: true },
      ]);

      render(<LeftSidebar {...mockProps} />);

      await waitFor(() => {
        expect(screen.getByText('MyFolder')).toBeInTheDocument();
      });

      fireEvent.click(screen.getByText('MyFolder'));
      expect(mockProps.navigateToPath).toHaveBeenCalledWith('C:\\Users\\Test\\MyFolder');
    });

    it('removes bookmark when remove button is clicked', async () => {
      vi.mocked(TauriAPI.getBookmarks).mockResolvedValueOnce([
        { name: 'MyFolder', path: 'C:\\Users\\Test\\MyFolder', is_dir: true },
      ]);

      render(<LeftSidebar {...mockProps} />);

      await waitFor(() => {
        expect(screen.getByText('MyFolder')).toBeInTheDocument();
      });

      const removeButton = screen.getByTitle('Remove bookmark');
      fireEvent.click(removeButton);

      expect(TauriAPI.removeBookmark).toHaveBeenCalledWith('C:\\Users\\Test\\MyFolder');
    });
  });

  describe('Drives Section', () => {
    it('renders drive list', async () => {
      render(<LeftSidebar {...mockProps} />);

      await waitFor(() => {
        expect(screen.getByText('C:')).toBeInTheDocument();
        expect(screen.getByText('D:')).toBeInTheDocument();
      });
    });

    it('navigates to drive when drive is clicked', async () => {
      render(<LeftSidebar {...mockProps} />);

      await waitFor(() => {
        expect(screen.getByText('C:')).toBeInTheDocument();
      });

      fireEvent.click(screen.getByText('C:'));
      expect(mockProps.navigateToPath).toHaveBeenCalledWith('C:\\');
    });
  });

  describe('Layout', () => {
    it('applies correct container styles', () => {
      const { container } = render(<LeftSidebar {...mockProps} />);

      const sidebar = container.firstChild as HTMLElement;
      expect(sidebar).toHaveClass(
        'bg-xp-surface',
        'border-r',
        'border-xp-border',
        'flex',
        'flex-col',
      );
    });

    it('applies default width when width prop not provided', () => {
      const { container } = render(<LeftSidebar {...mockProps} />);

      const sidebar = container.firstChild as HTMLElement;
      expect(sidebar.style.width).toBe('200px');
    });

    it('applies custom width when width prop provided', () => {
      const { container } = render(<LeftSidebar {...mockProps} width={300} />);

      const sidebar = container.firstChild as HTMLElement;
      expect(sidebar.style.width).toBe('300px');
    });

    it('forwards data-tour attribute', () => {
      const { container } = render(<LeftSidebar {...mockProps} data-tour="sidebar-tour" />);

      const sidebar = container.firstChild as HTMLElement;
      expect(sidebar.getAttribute('data-tour')).toBe('sidebar-tour');
    });
  });

  describe('Edge Cases', () => {
    it('handles getUserDirectories failure gracefully', async () => {
      vi.mocked(TauriAPI.getUserDirectories).mockRejectedValueOnce(new Error('Access denied'));

      expect(() => render(<LeftSidebar {...mockProps} />)).not.toThrow();

      await waitFor(() => {
        expect(screen.getByText('home')).toBeInTheDocument();
      });
    });

    it('handles listDrives failure gracefully', async () => {
      vi.mocked(TauriAPI.listDrives).mockRejectedValueOnce(new Error('Access denied'));

      expect(() => render(<LeftSidebar {...mockProps} />)).not.toThrow();
    });

    it('handles getBookmarks failure gracefully', async () => {
      vi.mocked(TauriAPI.getBookmarks).mockRejectedValueOnce(new Error('DB error'));

      expect(() => render(<LeftSidebar {...mockProps} />)).not.toThrow();
    });

    it('handles empty files array', () => {
      const emptyProps = { ...mockProps, files: [] };
      expect(() => render(<LeftSidebar {...emptyProps} />)).not.toThrow();
    });
  });
});
