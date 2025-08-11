import { describe, it, expect, beforeEach, vi } from 'vitest';
import { mockInvoke, resetAllMocks } from '../../test-utils';

vi.mock('@tauri-apps/api/core', () => ({
  invoke: mockInvoke,
}));

describe('Highlight Clear Integration', () => {
  beforeEach(() => {
    resetAllMocks();
  });

  it('should re-enable highlights when user types after clearing with ESC', async () => {
    const { createSearchManager } = await import('../../../lib/core/searchManager.svelte');
    const { createAppCoordinator } = await import('../../../lib/app/appCoordinator.svelte');
    const { createEditorManager } = await import('../../../lib/core/editorManager.svelte');
    const { createFocusManager } = await import('../../../lib/core/focusManager.svelte');

    // Create manager instances for testing
    const searchManager = createSearchManager();
    const editorManager = createEditorManager();
    const focusManager = createFocusManager();
    const appCoordinator = createAppCoordinator({
      searchManager,
      editorManager,
      focusManager
    });

    const contentManager = appCoordinator.context.contentManager;

    searchManager.searchInput = 'test query';
    contentManager.setHighlightsClearedState(false);
    expect(contentManager.areHighlightsCleared).toBe(false);

    contentManager.clearHighlights();
    expect(contentManager.areHighlightsCleared).toBe(true);

    searchManager.searchInput = 'new search';

    await new Promise(resolve => setTimeout(resolve, 0));

    expect(contentManager.areHighlightsCleared).toBe(false);
  });
});
