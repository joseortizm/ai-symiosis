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
    const { searchManager } = await import('../../../lib/utils/searchManager.svelte');
    const { appCoordinator } = await import('../../../lib/utils/appCoordinator.svelte');

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
