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
    // Reset service state by closing if open
    if (configService.isVisible) {
      configService.close();
    }
  });

  describe('open', () => {
    it('should load config content and show settings pane', async () => {
      const configContent = 'notes_directory = "/path/to/notes"';
      mockInvoke.mockResolvedValueOnce(configContent);

      const onFocus = vi.fn();

      await configService.open();

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
    it('should close settings pane and clear content', async () => {
      // Open service first to put it in visible state
      mockInvoke.mockResolvedValueOnce('some content');
      await configService.open();

      expect(configService.isVisible).toBe(true);
      expect(configService.content).toBe('some content');

      configService.close();

      expect(configService.isVisible).toBe(false);
      expect(configService.content).toBe('');
      expect(configService.error).toBeNull();
    });

    it('should work without onFocus callback', async () => {
      // Open service first to make it visible
      mockInvoke.mockResolvedValueOnce('test content');
      await configService.open();

      expect(() => configService.close()).not.toThrow();
      expect(configService.isVisible).toBe(false);
    });
  });

  describe('save', () => {
    it('should save config and refresh cache successfully', async () => {
      const configContent = 'notes_directory = "/new/path"';

      // Open service and set content
      mockInvoke.mockResolvedValueOnce(configContent);
      await configService.open();
      configService.content = configContent;

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

      // Open service and set content
      mockInvoke.mockResolvedValueOnce('some content');
      await configService.open();
      configService.content = 'some content';

      mockInvoke.mockRejectedValueOnce(error);

      const result = await configService.save();

      expect(result.success).toBe(false);
      expect(result.error).toBe('Failed to save config: Error: Permission denied');
      expect(configService.error).toBe('Failed to save config: Error: Permission denied');
      expect(configService.isLoading).toBe(false);
    });

    it('should track loading state during save', async () => {
      let loadingDuringOperation = false;

      // Open service and set content
      mockInvoke.mockResolvedValueOnce('test content');
      await configService.open();
      configService.content = 'test content';

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
    it('should clear errors', async () => {
      // Create an error by triggering a failed operation
      mockInvoke.mockRejectedValueOnce(new Error('Test error'));
      await configService.open();

      expect(configService.error).toBeTruthy();

      configService.clearError();

      expect(configService.error).toBeNull();
    });
  });

  describe('reactive state getters', () => {
    it('should return correct state values', async () => {
      // Test initial state
      expect(configService.isVisible).toBe(false);
      expect(configService.isLoading).toBe(false);
      expect(configService.error).toBeNull();

      // Open service to set it to visible state
      mockInvoke.mockResolvedValueOnce('test content');
      await configService.open();
      configService.content = 'test content';

      expect(configService.content).toBe('test content');
      expect(configService.isVisible).toBe(true);
      expect(configService.isLoading).toBe(false); // Loading is false after successful open
      expect(configService.error).toBeNull(); // No error after successful operation
    });
  });

  describe('pane management methods', () => {
    it('should open pane with focus management', async () => {
      const configContent = 'notes_directory = "/path/to/notes"';
      mockInvoke.mockResolvedValueOnce(configContent);
      const mockFocusFunction = vi.fn();

      await configService.openPane();

      expect(mockInvoke).toHaveBeenCalledWith('get_config_content');
      expect(configService.content).toBe(configContent);
      expect(configService.isVisible).toBe(true);
    });

    it('should close pane with focus management', async () => {
      // Open service first to make it visible
      mockInvoke.mockResolvedValueOnce('some content');
      await configService.openPane();

      expect(configService.isVisible).toBe(true);

      configService.closePane();

      expect(configService.isVisible).toBe(false);
      expect(configService.content).toBe('');
    });
  });
});
