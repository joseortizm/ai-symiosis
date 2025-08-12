import { describe, it, expect, beforeEach, vi } from 'vitest';

// Mock all the services and managers
const mockNoteService = {
  getContent: vi.fn()
};

const mockSearchManager = {
  searchInput: '',
  filteredNotes: [],
  isLoading: false,
  areHighlightsCleared: false,
  setFilteredNotes: vi.fn((notes) => { mockSearchManager.filteredNotes = notes; }),
  searchImmediate: vi.fn(),
  clearSearch: vi.fn()
};

const mockContentManager = {
  setNoteContent: vi.fn(),
  scrollToFirstMatch: vi.fn(),
  highlightedContent: '',
  clearHighlights: vi.fn()
};

const mockEditorManager = {
  isEditMode: false,
  exitEditMode: vi.fn()
};

const mockFocusManager = {
  selectedIndex: -1,
  setSelectedIndex: vi.fn((index) => { mockFocusManager.selectedIndex = index; }),
  focusSearch: vi.fn()
};

const mockDialogManager = {
  showCreateDialog: false,
  showDeleteDialog: false,
  showRenameDialog: false,
  openCreateDialog: vi.fn(),
  openDeleteDialog: vi.fn(),
  openRenameDialog: vi.fn()
};

// Mock all the modules
vi.mock('../../../lib/services/noteService.svelte', () => ({
  noteService: mockNoteService
}));

vi.mock('../../../lib/core/searchManager.svelte', () => ({
  createSearchManager: () => mockSearchManager
}));

vi.mock('../../../lib/core/editorManager.svelte', () => ({
  createEditorManager: () => mockEditorManager
}));

vi.mock('../../../lib/core/focusManager.svelte', () => ({
  createFocusManager: () => mockFocusManager
}));

vi.mock('../../../lib/core/dialogManager.svelte', () => ({
  createDialogManager: () => mockDialogManager
}));

vi.mock('../../../lib/core/contentManager.svelte', () => ({
  createContentManager: () => mockContentManager
}));

const { createAppCoordinator } = await import('../../../lib/app/appCoordinator.svelte');
const { createKeyboardActions } = await import('../../../lib/app/actions/keyboard.svelte');

describe('Content Loading Integration', () => {
  let appCoordinator: ReturnType<typeof createAppCoordinator>;

  beforeEach(() => {
    vi.clearAllMocks();
    mockFocusManager.selectedIndex = -1;
    mockSearchManager.filteredNotes = [];
    mockSearchManager.searchInput = '';
    mockSearchManager.isLoading = false;
    mockSearchManager.areHighlightsCleared = false;
    mockEditorManager.isEditMode = false;

    appCoordinator = createAppCoordinator({
      searchManager: mockSearchManager as any,
      editorManager: mockEditorManager as any,
      focusManager: mockFocusManager as any
    });
  });

  describe('selectNote action', () => {
    it('should load content when note is selected via mouse click', async () => {
      const testContent = 'Test note content';
      mockNoteService.getContent.mockResolvedValue(testContent);

      await appCoordinator.actions.selectNote('test-note.md', 0);

      expect(mockFocusManager.setSelectedIndex).toHaveBeenCalledWith(0);
      expect(mockNoteService.getContent).toHaveBeenCalledWith('test-note.md');
      expect(mockContentManager.setNoteContent).toHaveBeenCalledWith(testContent);
    });

    it('should handle content loading errors gracefully', async () => {
      const error = new Error('Failed to load content');
      mockNoteService.getContent.mockRejectedValue(error);

      await appCoordinator.actions.selectNote('error-note.md', 1);

      expect(mockFocusManager.setSelectedIndex).toHaveBeenCalledWith(1);
      expect(mockNoteService.getContent).toHaveBeenCalledWith('error-note.md');
      expect(mockContentManager.setNoteContent).toHaveBeenCalledWith('Error loading note: Error: Failed to load content');
    });

    it('should clear content when no note is provided', async () => {
      await appCoordinator.actions.selectNote('', -1);

      expect(mockNoteService.getContent).not.toHaveBeenCalled();
      expect(mockContentManager.setNoteContent).toHaveBeenCalledWith('');
    });

    it('should abort previous requests when selecting new note', async () => {
      const controller1 = new AbortController();
      const controller2 = new AbortController();
      let resolveFirst: (value: string) => void;
      let resolveSecond: (value: string) => void;

      const firstPromise = new Promise<string>(resolve => { resolveFirst = resolve; });
      const secondPromise = new Promise<string>(resolve => { resolveSecond = resolve; });

      mockNoteService.getContent
        .mockReturnValueOnce(firstPromise)
        .mockReturnValueOnce(secondPromise);

      const firstSelection = appCoordinator.actions.selectNote('first.md', 0);
      const secondSelection = appCoordinator.actions.selectNote('second.md', 1);

      resolveFirst('First content');
      resolveSecond('Second content');

      await Promise.all([firstSelection, secondSelection]);

      expect(mockContentManager.setNoteContent).toHaveBeenCalledWith('Second content');
      expect(mockContentManager.setNoteContent).toHaveBeenCalledTimes(1);
    });

    it('should not update selectedIndex if it is already correct', async () => {
      mockFocusManager.selectedIndex = 2;
      mockNoteService.getContent.mockResolvedValue('Content');

      await appCoordinator.actions.selectNote('test.md', 2);

      expect(mockFocusManager.setSelectedIndex).not.toHaveBeenCalled();
      expect(mockNoteService.getContent).toHaveBeenCalledWith('test.md');
    });

    it('should trigger scrollToFirstMatch after content loads', async () => {
      mockNoteService.getContent.mockResolvedValue('Content with search terms');

      await appCoordinator.actions.selectNote('test.md', 0);

      await new Promise(resolve => requestAnimationFrame(resolve));

      expect(mockContentManager.scrollToFirstMatch).toHaveBeenCalled();
    });
  });

  describe('keyboard navigation integration', () => {
    let keyboardActions: ReturnType<typeof createKeyboardActions>;

    beforeEach(() => {
      mockSearchManager.filteredNotes = ['note1.md', 'note2.md', 'note3.md'];

      keyboardActions = createKeyboardActions({
        focusManager: mockFocusManager,
        selectNote: appCoordinator.actions.selectNote,
        enterEditMode: vi.fn(),
        exitEditMode: vi.fn(),
        saveAndExitNote: vi.fn(),
        showExitEditDialog: vi.fn(),
        showDeleteDialog: vi.fn(),
        showCreateDialog: vi.fn(),
        showRenameDialog: vi.fn(),
        openSettingsPane: vi.fn(),
        clearHighlights: vi.fn(),
        clearSearch: vi.fn(),
        focusSearch: vi.fn()
      });
    });

    it('should load content when keyboard navigation calls moveUp action', async () => {
      mockFocusManager.selectedIndex = 2;
      mockNoteService.getContent.mockResolvedValue('Note 2 content');

      const mockState = {
        isSearchInputFocused: false,
        isEditMode: false,
        isNoteContentFocused: false,
        filteredNotes: ['note1.md', 'note2.md', 'note3.md'],
        selectedNote: 'note3.md',
        noteContentElement: null,
        areHighlightsCleared: false,
        isEditorDirty: false,
        query: ''
      };

      const mockActionContext = {
        state: mockState,
        actions: {
          focusManager: mockFocusManager,
          selectNote: appCoordinator.actions.selectNote
        }
      };

      keyboardActions.actionRegistry.navigation.moveUp(mockActionContext as any);

      // Wait for async content loading to complete
      await new Promise(resolve => setTimeout(resolve, 0));

      expect(mockNoteService.getContent).toHaveBeenCalledWith('note2.md');
      expect(mockContentManager.setNoteContent).toHaveBeenCalledWith('Note 2 content');
    });

    it('should load content when keyboard navigation calls moveDown action', async () => {
      mockFocusManager.selectedIndex = 0;
      mockNoteService.getContent.mockResolvedValue('Note 2 content');

      const mockState = {
        isSearchInputFocused: false,
        isEditMode: false,
        isNoteContentFocused: false,
        filteredNotes: ['note1.md', 'note2.md', 'note3.md'],
        selectedNote: 'note1.md',
        noteContentElement: null,
        areHighlightsCleared: false,
        isEditorDirty: false,
        query: ''
      };

      const mockActionContext = {
        state: mockState,
        actions: {
          focusManager: mockFocusManager,
          selectNote: appCoordinator.actions.selectNote
        }
      };

      keyboardActions.actionRegistry.navigation.moveDown(mockActionContext as any);

      // Wait for async content loading to complete
      await new Promise(resolve => setTimeout(resolve, 0));

      expect(mockNoteService.getContent).toHaveBeenCalledWith('note2.md');
      expect(mockContentManager.setNoteContent).toHaveBeenCalledWith('Note 2 content');
    });

    it('should handle boundary conditions in keyboard navigation', async () => {
      mockFocusManager.selectedIndex = 0;
      mockNoteService.getContent.mockResolvedValue('Note 1 content');

      const mockState = {
        filteredNotes: ['note1.md', 'note2.md', 'note3.md']
      };

      const mockActionContext = {
        state: mockState,
        actions: {
          focusManager: mockFocusManager,
          selectNote: appCoordinator.actions.selectNote
        }
      };

      keyboardActions.actionRegistry.navigation.moveUp(mockActionContext as any);

      // Wait for async content loading to complete
      await new Promise(resolve => setTimeout(resolve, 0));

      expect(mockNoteService.getContent).toHaveBeenCalledWith('note1.md');
      expect(mockContentManager.setNoteContent).toHaveBeenCalledWith('Note 1 content');
    });
  });

  describe('selectNote action comprehensive behavior', () => {
    it('should handle consistent content loading between direct calls and keyboard actions', async () => {
      const testContent = 'Consistent content';
      mockNoteService.getContent.mockResolvedValue(testContent);

      await appCoordinator.actions.selectNote('note2.md', 1);
      expect(mockContentManager.setNoteContent).toHaveBeenCalledWith(testContent);
      expect(mockFocusManager.setSelectedIndex).toHaveBeenCalledWith(1);

      vi.clearAllMocks();
      mockNoteService.getContent.mockResolvedValue(testContent);

      const keyboardActions = createKeyboardActions({
        focusManager: mockFocusManager,
        selectNote: appCoordinator.actions.selectNote,
        enterEditMode: vi.fn(),
        exitEditMode: vi.fn(),
        saveAndExitNote: vi.fn(),
        showExitEditDialog: vi.fn(),
        showDeleteDialog: vi.fn(),
        showCreateDialog: vi.fn(),
        showRenameDialog: vi.fn(),
        openSettingsPane: vi.fn(),
        clearHighlights: vi.fn(),
        clearSearch: vi.fn(),
        focusSearch: vi.fn()
      });

      mockFocusManager.selectedIndex = 0;
      const mockActionContext = {
        state: { filteredNotes: ['note1.md', 'note2.md', 'note3.md'] },
        actions: { focusManager: mockFocusManager, selectNote: appCoordinator.actions.selectNote }
      };

      keyboardActions.actionRegistry.navigation.moveDown(mockActionContext as any);

      // Wait for async content loading to complete
      await new Promise(resolve => setTimeout(resolve, 0));

      expect(mockContentManager.setNoteContent).toHaveBeenCalledWith(testContent);
      expect(mockNoteService.getContent).toHaveBeenCalledWith('note2.md');
    });
  });
});
