import { describe, it, expect, beforeEach, vi, afterEach } from 'vitest';
import { createConfigStateManager } from '$lib/core/configStateManager.svelte';

// Mock Tauri APIs
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn()
}));

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn()
}));

describe('configStateManager', () => {
  let manager: ReturnType<typeof createConfigStateManager>;
  let mockUnlisten: ReturnType<typeof vi.fn>;
  let mockInvoke: ReturnType<typeof vi.fn>;
  let mockListen: ReturnType<typeof vi.fn>;

  beforeEach(async () => {
    vi.clearAllMocks();
    
    // Get the mocked functions
    const { invoke } = await import('@tauri-apps/api/core');
    const { listen } = await import('@tauri-apps/api/event');
    mockInvoke = invoke as ReturnType<typeof vi.fn>;
    mockListen = listen as ReturnType<typeof vi.fn>;
    
    mockUnlisten = vi.fn();
    mockListen.mockResolvedValue(mockUnlisten);
    manager = createConfigStateManager();
  });

  afterEach(() => {
    manager.cleanup();
  });

  describe('initial state', () => {
    it('should have default values before initialization', () => {
      expect(manager.notesDirectory).toBe('');
      expect(manager.maxSearchResults).toBe(100);
      expect(manager.globalShortcut).toBe('Ctrl+Shift+N');
      expect(manager.editorMode).toBe('basic');
      expect(manager.markdownTheme).toBe('dark_dimmed');
      expect(manager.isLoading).toBe(false);
      expect(manager.error).toBe(null);
      expect(manager.isInitialized).toBe(false);
    });
  });

  describe('initialize', () => {
    it('should load initial config values successfully', async () => {
      mockInvoke
        .mockResolvedValueOnce('vim') // get_editor_mode
        .mockResolvedValueOnce('github'); // get_markdown_theme

      await manager.initialize();

      expect(mockInvoke).toHaveBeenCalledWith('get_editor_mode');
      expect(mockInvoke).toHaveBeenCalledWith('get_markdown_theme');
      expect(mockListen).toHaveBeenCalledWith('config-changed', expect.any(Function));
      
      expect(manager.editorMode).toBe('vim');
      expect(manager.markdownTheme).toBe('github');
      expect(manager.isInitialized).toBe(true);
      expect(manager.isLoading).toBe(false);
      expect(manager.error).toBe(null);
    });

    it('should handle initialization errors gracefully', async () => {
      const errorMessage = 'Failed to get config';
      mockInvoke.mockRejectedValue(new Error(errorMessage));

      await manager.initialize();

      expect(manager.error).toContain('Failed to initialize config state');
      expect(manager.error).toContain(errorMessage);
      expect(manager.isInitialized).toBe(false);
      expect(manager.isLoading).toBe(false);
    });

    it('should not initialize twice', async () => {
      mockInvoke
        .mockResolvedValueOnce('vim')
        .mockResolvedValueOnce('github');

      await manager.initialize();
      expect(mockInvoke).toHaveBeenCalledTimes(2);

      // Call initialize again
      await manager.initialize();
      expect(mockInvoke).toHaveBeenCalledTimes(2); // Should not call again
    });

    it('should set loading states correctly during initialization', async () => {
      mockInvoke
        .mockResolvedValueOnce('vim')
        .mockResolvedValueOnce('github');

      expect(manager.isLoading).toBe(false);
      
      const initPromise = manager.initialize();
      expect(manager.isLoading).toBe(true);
      
      await initPromise;
      expect(manager.isLoading).toBe(false);
    });
  });

  describe('config change events', () => {
    it('should update state when config-changed event is received', async () => {
      let configChangeHandler: ((event: any) => void) | null = null;
      
      mockListen.mockImplementation((eventName, handler) => {
        if (eventName === 'config-changed') {
          configChangeHandler = handler;
        }
        return Promise.resolve(mockUnlisten);
      });

      mockInvoke
        .mockResolvedValueOnce('basic')
        .mockResolvedValueOnce('dark_dimmed');

      await manager.initialize();

      // Simulate config change event
      const newConfig = {
        notes_directory: '/new/path',
        max_search_results: 200,
        global_shortcut: 'Ctrl+Alt+N',
        editor_mode: 'vim',
        markdown_theme: 'github'
      };

      expect(configChangeHandler).not.toBeNull();
      configChangeHandler!({ payload: newConfig });

      expect(manager.notesDirectory).toBe('/new/path');
      expect(manager.maxSearchResults).toBe(200);
      expect(manager.globalShortcut).toBe('Ctrl+Alt+N');
      expect(manager.editorMode).toBe('vim');
      expect(manager.markdownTheme).toBe('github');
    });
  });

  describe('forceRefresh', () => {
    beforeEach(async () => {
      mockInvoke
        .mockResolvedValueOnce('basic')
        .mockResolvedValueOnce('dark_dimmed');
      await manager.initialize();
      vi.clearAllMocks();
    });

    it('should refresh config values from backend', async () => {
      mockInvoke
        .mockResolvedValueOnce(undefined) // refresh_cache
        .mockResolvedValueOnce('vim') // get_editor_mode
        .mockResolvedValueOnce('github'); // get_markdown_theme

      await manager.forceRefresh();

      expect(mockInvoke).toHaveBeenCalledWith('refresh_cache');
      expect(mockInvoke).toHaveBeenCalledWith('get_editor_mode');
      expect(mockInvoke).toHaveBeenCalledWith('get_markdown_theme');
      
      expect(manager.editorMode).toBe('vim');
      expect(manager.markdownTheme).toBe('github');
      expect(manager.error).toBe(null);
    });

    it('should handle refresh errors gracefully', async () => {
      const errorMessage = 'Refresh failed';
      mockInvoke.mockRejectedValue(new Error(errorMessage));

      await manager.forceRefresh();

      expect(manager.error).toContain('Failed to refresh config');
      expect(manager.error).toContain(errorMessage);
      expect(manager.isLoading).toBe(false);
    });

    it('should set loading states correctly during refresh', async () => {
      mockInvoke
        .mockResolvedValueOnce(undefined)
        .mockResolvedValueOnce('vim')
        .mockResolvedValueOnce('github');

      expect(manager.isLoading).toBe(false);
      
      const refreshPromise = manager.forceRefresh();
      expect(manager.isLoading).toBe(true);
      
      await refreshPromise;
      expect(manager.isLoading).toBe(false);
    });

    it('should clear previous errors on refresh', async () => {
      // First, cause an error
      mockInvoke.mockRejectedValueOnce(new Error('Initial error'));
      await manager.forceRefresh();
      expect(manager.error).toBeTruthy();

      // Then refresh successfully
      mockInvoke
        .mockResolvedValueOnce(undefined)
        .mockResolvedValueOnce('vim')
        .mockResolvedValueOnce('github');

      await manager.forceRefresh();
      expect(manager.error).toBe(null);
    });
  });

  describe('cleanup', () => {
    it('should cleanup event listeners and reset initialization state', async () => {
      mockInvoke
        .mockResolvedValueOnce('vim')
        .mockResolvedValueOnce('github');

      await manager.initialize();
      expect(manager.isInitialized).toBe(true);

      manager.cleanup();

      expect(mockUnlisten).toHaveBeenCalled();
      expect(manager.isInitialized).toBe(false);
    });

    it('should handle cleanup when not initialized', () => {
      expect(() => manager.cleanup()).not.toThrow();
      expect(manager.isInitialized).toBe(false);
    });

    it('should handle cleanup multiple times', async () => {
      mockInvoke
        .mockResolvedValueOnce('vim')
        .mockResolvedValueOnce('github');

      await manager.initialize();
      
      manager.cleanup();
      expect(() => manager.cleanup()).not.toThrow();
    });
  });

  describe('reactive getters', () => {
    it('should provide reactive access to all config properties', () => {
      const properties = [
        'notesDirectory',
        'maxSearchResults', 
        'globalShortcut',
        'editorMode',
        'markdownTheme',
        'isLoading',
        'error',
        'isInitialized'
      ];

      properties.forEach(prop => {
        expect(manager).toHaveProperty(prop);
        expect(typeof (manager as any)[prop]).not.toBe('function');
      });
    });
  });

  describe('error handling', () => {
    it('should handle partial initialization failures', async () => {
      mockInvoke
        .mockResolvedValueOnce('vim') // get_editor_mode succeeds
        .mockRejectedValueOnce(new Error('Theme fetch failed')); // get_markdown_theme fails

      await manager.initialize();

      expect(manager.error).toContain('Failed to initialize config state');
      expect(manager.isInitialized).toBe(false);
    });

    it('should handle event listener setup failures', async () => {
      mockInvoke
        .mockResolvedValueOnce('vim')
        .mockResolvedValueOnce('github');
      
      mockListen.mockRejectedValue(new Error('Event listener failed'));

      await manager.initialize();

      expect(manager.error).toContain('Failed to initialize config state');
      expect(manager.isInitialized).toBe(false);
    });
  });
});