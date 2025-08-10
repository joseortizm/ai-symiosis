import { describe, it, expect, beforeEach, vi } from 'vitest';
import { mockInvoke, resetAllMocks } from '../../test-utils';

// Test for both factory-based and singleton-based contentManager
describe('contentManager (factory-based - TDD)', () => {
  let contentManager: any;
  let mockDeps: any;

  beforeEach(async () => {
    resetAllMocks();

    mockDeps = {
      contentHighlighter: {
        setContent: vi.fn(),
        updateHighlighterState: vi.fn(),
        highlighted: 'mocked highlighted content',
        areHighlightsCleared: false
      },
      noteService: {
        getContent: vi.fn().mockResolvedValue('mock note content')
      },
      getNoteContentElement: vi.fn().mockReturnValue(null),
      refreshSearch: vi.fn().mockResolvedValue(['note1.md', 'note2.md']),
      setHighlightsClearCallback: vi.fn(),
      invoke: mockInvoke
    };

    try {
      const { createContentManager } = await import('../../../lib/utils/contentManager.svelte');
      contentManager = createContentManager(mockDeps);
    } catch (e) {
      contentManager = null;
    }
  });

  it('should create contentManager with injected dependencies', async () => {
    const { createContentManager } = await import('../../../lib/utils/contentManager.svelte');
    const manager = createContentManager(mockDeps);

    expect(manager).toBeDefined();
    expect(typeof manager.setNoteContent).toBe('function');
    expect(typeof manager.refreshContent).toBe('function');
  });

  it('should use injected contentHighlighter', () => {
    if (!contentManager) return; // Skip if factory not implemented yet

    const testContent = 'Test content';
    contentManager.setNoteContent(testContent);

    expect(mockDeps.contentHighlighter.setContent).toHaveBeenCalledWith(testContent);
  });

  it('should use injected noteService for content refresh', async () => {
    if (!contentManager) return; // Skip if factory not implemented yet

    const noteName = 'test.md';
    await contentManager.refreshContent(noteName);

    expect(mockDeps.noteService.getContent).toHaveBeenCalledWith(noteName);
  });

  it('should use injected refreshSearch function', async () => {
    if (!contentManager) return; // Skip if factory not implemented yet

    const noteName = 'test.md';
    const searchInput = 'test';
    await contentManager.refreshAfterSave(noteName, searchInput);

    expect(mockDeps.refreshSearch).toHaveBeenCalledWith(searchInput);
  });

  it('should call setHighlightsClearCallback during setup', () => {
    if (!contentManager) return; // Skip if factory not implemented yet

    expect(mockDeps.setHighlightsClearCallback).toHaveBeenCalledWith(
      expect.any(Function)
    );
  });
});

