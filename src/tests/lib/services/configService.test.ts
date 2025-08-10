import { describe, it, expect, beforeEach, vi } from 'vitest';
import { mockInvoke, mockSearchManager, resetAllMocks } from '../../test-utils';

vi.mock('@tauri-apps/api/core', () => ({
  invoke: mockInvoke,
}));

// Import after mocking
const { configService } = await import('../../../lib/services/configService.svelte');

describe('configService', () => {
  beforeEach(() => {
    resetAllMocks();
    configService.clearError();
    configService.state.isVisible = false;
    configService.state.content = '';
  });

  describe('open', () => {
    it('should load config content and show settings pane', async () => {
      const configContent = 'notes_directory = "/path/to/notes"';
      mockInvoke.mockResolvedValueOnce(configContent);

      const onFocus = vi.fn();

      await configService.open(onFocus);

      expect(mockInvoke).toHaveBeenCalledWith('get_config_content');
      expect(configService.content).toBe(configContent);
      expect(configService.isVisible).toBe(true);
      expect(configService.isLoading).toBe(false);
      expect(configService.error).toBeNull();
    });

    it('should handle errors when loading config', async () => {
      const error = new Error('Config not found');
      mockInvoke.mockRejectedValueOnce(error);

      await configService.open();

      expect(configService.error).toBe('Failed to load config: Error: Config not found');
      expect(configService.isVisible).toBe(false);
      expect(configService.isLoading).toBe(false);
    });

    it('should track loading state during open', async () => {
      let loadingDuringOperation = false;
      mockInvoke.mockImplementation(() => {
        loadingDuringOperation = configService.isLoading;
        return Promise.resolve('config content');
      });

      await configService.open();

      expect(loadingDuringOperation).toBe(true);
      expect(configService.isLoading).toBe(false);
    });
  });

  describe('close', () => {
    it('should close settings pane and clear content', () => {
      configService.state.isVisible = true;
      configService.state.content = 'some content';
      configService.state.error = 'some error';

      configService.close();

      expect(configService.isVisible).toBe(false);
      expect(configService.content).toBe('');
      expect(configService.error).toBeNull();
    });

    it('should work without onFocus callback', () => {
      configService.state.isVisible = true;

      expect(() => configService.close()).not.toThrow();
      expect(configService.isVisible).toBe(false);
    });
  });

  describe('save', () => {
    it('should save config and refresh cache successfully', async () => {
      const configContent = 'notes_directory = "/new/path"';
      configService.state.content = configContent;
      configService.state.isVisible = true;

      mockInvoke
        .mockResolvedValueOnce(undefined) // save_config_content
        .mockResolvedValueOnce(undefined); // refresh_cache

      const result = await configService.save();

      expect(result.success).toBe(true);
      expect(mockInvoke).toHaveBeenCalledWith('save_config_content', { content: configContent });
      expect(mockInvoke).toHaveBeenCalledWith('refresh_cache');
      expect(configService.isVisible).toBe(false);
      expect(configService.content).toBe('');
      expect(configService.isLoading).toBe(false);
      expect(configService.error).toBeNull();
    });

    it('should handle save errors', async () => {
      const error = new Error('Permission denied');
      configService.state.content = 'some content';
      mockInvoke.mockRejectedValueOnce(error);

      const result = await configService.save();

      expect(result.success).toBe(false);
      expect(result.error).toBe('Failed to save config: Error: Permission denied');
      expect(configService.error).toBe('Failed to save config: Error: Permission denied');
      expect(configService.isLoading).toBe(false);
    });

    it('should track loading state during save', async () => {
      let loadingDuringOperation = false;
      configService.state.content = 'test content';

      mockInvoke.mockImplementation(() => {
        loadingDuringOperation = configService.isLoading;
        return Promise.resolve();
      });

      await configService.save();

      expect(loadingDuringOperation).toBe(true);
      expect(configService.isLoading).toBe(false);
    });
  });

  describe('content management', () => {
    it('should update content', () => {
      const newContent = 'new config content';

      configService.updateContent(newContent);

      expect(configService.content).toBe(newContent);
    });

    it('should support getter/setter for content', () => {
      const content = 'test content';

      configService.content = content;

      expect(configService.content).toBe(content);
      expect(configService.state.content).toBe(content);
    });
  });

  describe('utility methods', () => {
    it('should check if config exists', async () => {
      mockInvoke.mockResolvedValueOnce(true);

      const exists = await configService.exists();

      expect(mockInvoke).toHaveBeenCalledWith('config_exists');
      expect(exists).toBe(true);
    });

    it('should handle errors when checking config existence', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Access denied'));

      const exists = await configService.exists();

      expect(exists).toBe(false);
    });

    it('should refresh cache manually', async () => {
      mockInvoke.mockResolvedValueOnce(undefined);

      await configService.refreshCache();

      expect(mockInvoke).toHaveBeenCalledWith('refresh_cache');
    });

    it('should handle refresh cache errors', async () => {
      const error = new Error('Cache refresh failed');
      mockInvoke.mockRejectedValueOnce(error);

      await expect(configService.refreshCache()).rejects.toThrow(error);
    });
  });

  describe('error handling', () => {
    it('should clear errors', () => {
      configService.state.error = 'Some error';

      configService.clearError();

      expect(configService.error).toBeNull();
    });
  });

  describe('reactive state getters', () => {
    it('should return correct state values', () => {
      configService.state.content = 'test content';
      configService.state.isVisible = true;
      configService.state.isLoading = true;
      configService.state.error = 'test error';

      expect(configService.content).toBe('test content');
      expect(configService.isVisible).toBe(true);
      expect(configService.isLoading).toBe(true);
      expect(configService.error).toBe('test error');
    });
  });

  describe('pane management methods', () => {
    it('should open pane with focus management', async () => {
      const configContent = 'notes_directory = "/path/to/notes"';
      mockInvoke.mockResolvedValueOnce(configContent);
      const mockFocusFunction = vi.fn();

      await configService.openPane(mockFocusFunction);

      expect(mockInvoke).toHaveBeenCalledWith('get_config_content');
      expect(configService.content).toBe(configContent);
      expect(configService.isVisible).toBe(true);
    });

    it('should close pane with focus management', () => {
      configService.state.isVisible = true;
      configService.state.content = 'some content';

      configService.closePane();

      expect(configService.isVisible).toBe(false);
      expect(configService.content).toBe('');
    });
  });
});
