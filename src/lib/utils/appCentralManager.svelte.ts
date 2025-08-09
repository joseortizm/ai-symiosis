import { invoke } from "@tauri-apps/api/core";
import { tick } from "svelte";
import { listen } from "@tauri-apps/api/event";
import { searchManager } from './searchManager.svelte';
import { dialogManager } from './dialogManager.svelte';
import { noteService } from '../services/noteService.svelte';
import { configService } from '../services/configService.svelte';
import { editorManager } from './editorManager.svelte';
import { focusManager } from './focusManager.svelte';
import { contentManager } from './contentManager.svelte';

interface CentralAppState {
  searchInput: string;
  query: string;
  isLoading: boolean;
  areHighlightsCleared: boolean;
  filteredNotes: string[];
  selectedNote: string | null;
  selectedIndex: number;
}

const state = $state<CentralAppState>({
  searchInput: '',
  query: '',
  isLoading: false,
  areHighlightsCleared: false,
  filteredNotes: [],
  selectedNote: null,
  selectedIndex: -1,
});

let contentRequestController: AbortController | null = null;

function arraysEqual(a: string[], b: string[]): boolean {
  return a.length === b.length && a.every((val, i) => val === b[i]);
}

// Reactive coordination effects - these will be called from a component context
function setupReactiveEffects() {
  $effect(() => {
    searchManager.updateSearchInputWithEffects(
      state.searchInput,
      (query) => {
        state.query = query;
      },
      (cleared) => {
        state.areHighlightsCleared = cleared;
      }
    );
  });

  $effect(() => {
    state.isLoading = searchManager.isLoading;
  });

  $effect(() => {
    const notes = searchManager.filteredNotes;
    if (!arraysEqual(notes, state.filteredNotes)) {
      state.filteredNotes = notes;
      if (notes.length === 0) {
        state.selectedIndex = -1;
      } else {
        state.selectedIndex = 0;
      }
    }
  });

  $effect(() => {
    const newSelectedNote = state.filteredNotes.length > 0 && state.selectedIndex !== -1
      ? state.filteredNotes[state.selectedIndex]
      : null;
    if (newSelectedNote !== state.selectedNote) {
      state.selectedNote = newSelectedNote;
      editorManager.exitEditMode();
    }
  });

  $effect(() => {
    if (state.selectedIndex >= 0) {
      requestAnimationFrame(() => {
        focusManager.scrollToSelected(state.selectedIndex);
      });
    }
  });

  $effect(() => {
    if (!state.selectedNote) {
      contentManager.setNoteContent('');
      return;
    }

    if (contentRequestController) {
      contentRequestController.abort();
    }
    contentRequestController = new AbortController();
    const currentController = contentRequestController;

    (async () => {
      try {
        const content = await contentManager.getNoteContent(state.selectedNote!);

        if (!currentController.signal.aborted) {
          contentManager.setNoteContent(content);

          requestAnimationFrame(() => {
            contentManager.scrollToFirstMatch();
          });
        }
      } catch (e) {
        if (!currentController.signal.aborted) {
          console.error("Failed to load note content:", e);
          contentManager.setNoteContent(`Error loading note: ${e}`);
        }
      }
    })();
  });

  $effect(() => {
    contentManager.updateHighlighterState({
      query: state.query,
      areHighlightsCleared: state.areHighlightsCleared
    });
  });

  $effect(() => {
    dialogManager.updateState({
      selectedNote: state.selectedNote,
      query: state.query,
      highlightedContent: contentManager.highlightedContent,
      searchElement: focusManager.searchElement
    });
  });
}

// State setters
function setSearchInput(value: string): void {
  state.searchInput = value;
}

function setSelectedIndex(value: number): void {
  state.selectedIndex = value;
}

// Business logic functions
async function deleteNote(): Promise<void> {
  if (!state.selectedNote) return;

  await noteService.delete(
    state.selectedNote,
    searchManager,
    dialogManager,
    (notes) => { state.filteredNotes = notes; },
    state.searchInput,
    () => focusManager.focusSearch()
  );
}

async function createNote(noteNameParam?: string): Promise<void> {
  const inputNoteName = noteNameParam || dialogManager.newNoteName.trim();
  if (!inputNoteName.trim()) return;

  const finalNoteName = await noteService.create(
    inputNoteName,
    searchManager,
    dialogManager,
    (notes) => {
      state.filteredNotes = notes;
      // Select the new note
      const noteIndex = notes.findIndex(note => note === (inputNoteName.includes('.') ? inputNoteName : `${inputNoteName}.md`));
      if (noteIndex >= 0) {
        state.selectedIndex = noteIndex;
      }
    },
    () => focusManager.focusSearch()
  );
}

async function renameNote(newNameParam?: string): Promise<void> {
  const inputNewName = newNameParam || dialogManager.newNoteNameForRename.trim();
  if (!inputNewName.trim() || !state.selectedNote) return;

  await noteService.rename(
    state.selectedNote,
    inputNewName,
    searchManager,
    dialogManager,
    (notes) => { state.filteredNotes = notes; },
    (noteName) => {
      const noteIndex = state.filteredNotes.findIndex(note => note === noteName);
      if (noteIndex >= 0) {
        state.selectedIndex = noteIndex;
      }
    },
    state.searchInput
  );
}

async function saveNote(): Promise<void> {
  if (!state.selectedNote) return;

  const result = await editorManager.saveAndExit(state.selectedNote);

  if (result.success) {
    try {
      const refreshResult = await contentManager.refreshAfterSave(state.selectedNote, state.searchInput);
      state.filteredNotes = refreshResult.searchResults;

      await tick();
      focusManager.focusSearch();
    } catch (e) {
      console.error("Failed to refresh after save:", e);
    }
  } else {
    console.error("Failed to save note:", result.error);
  }
}

function selectNote(note: string, index: number): void {
  if (state.selectedIndex !== index) {
    state.selectedIndex = index;
  }
}

async function enterEditMode(): Promise<void> {
  if (state.selectedNote) {
    await editorManager.enterEditMode(
      state.selectedNote,
      contentManager.noteContent,
      focusManager.noteContentElement || undefined
    );
  }
}

function exitEditMode(): void {
  editorManager.exitEditMode();
  focusManager.focusSearch();
}

export const appCentralManager = {
  // Setup method for reactive effects
  setupReactiveEffects,
  // Reactive getters for state
  get searchInput(): string {
    return state.searchInput;
  },

  get query(): string {
    return state.query;
  },

  get isLoading(): boolean {
    return state.isLoading;
  },

  get areHighlightsCleared(): boolean {
    return state.areHighlightsCleared;
  },

  get filteredNotes(): string[] {
    return state.filteredNotes;
  },

  get selectedNote(): string | null {
    return state.selectedNote;
  },

  get selectedIndex(): number {
    return state.selectedIndex;
  },

  // State setters
  setSearchInput,

  // Add an update method for external state updates from components
  updateFilteredNotes(notes: string[]): void {
    state.filteredNotes = notes;
  },

  // Reset state for testing
  resetState(): void {
    state.searchInput = '';
    state.query = '';
    state.isLoading = false;
    state.areHighlightsCleared = false;
    state.filteredNotes = [];
    state.selectedNote = null;
    state.selectedIndex = -1;
    if (contentRequestController) {
      contentRequestController.abort();
      contentRequestController = null;
    }
  },

  setSelectedIndex,

  // Business logic functions
  deleteNote,
  createNote,
  renameNote,
  saveNote,
  selectNote,
  enterEditMode,
  exitEditMode,

  // Keyboard handler state aggregation
  get keyboardState() {
    return {
      isSearchInputFocused: focusManager.isSearchInputFocused,
      isEditMode: editorManager.isEditMode,
      isNoteContentFocused: focusManager.isNoteContentFocused,
      selectedIndex: state.selectedIndex,
      filteredNotes: state.filteredNotes,
      selectedNote: state.selectedNote,
      noteContentElement: focusManager.noteContentElement,
      searchElement: focusManager.searchElement,
      query: state.query,
      areHighlightsCleared: state.areHighlightsCleared,
      isEditorDirty: editorManager.isDirty,
    };
  },

  // Keyboard handler actions
  get keyboardActions() {
    return {
      setSelectedIndex,
      enterEditMode,
      exitEditMode,
      showExitEditDialog: dialogManager.showExitEditDialog,
      saveNote,
      invoke,
      showDeleteDialog: () => dialogManager.openDeleteDialog(),
      showCreateDialog: () => dialogManager.openCreateDialog(),
      showRenameDialog: () => dialogManager.openRenameDialog(),
      openSettingsPane: () => configService.openPane(() => focusManager.focusSearch()),
      clearHighlights: contentManager.clearHighlights,
      clearSearch: searchManager.clearSearch,
    };
  },

  // Context provider
  get context() {
    return {
      state: {
        get searchInput() { return state.searchInput; },
        set searchInput(value: string) { setSearchInput(value); },
        get query() { return state.query; },
        get isLoading() { return state.isLoading; },
        get areHighlightsCleared() { return state.areHighlightsCleared; },
        get filteredNotes() { return state.filteredNotes; },
        get selectedNote() { return state.selectedNote; },
        get selectedIndex() { return state.selectedIndex; },
      },
      editorManager,
      focusManager,
      contentManager,
      selectNote,
      deleteNote,
      createNote,
      renameNote,
      saveNote,
      enterEditMode,
      exitEditMode,
      showExitEditDialog: dialogManager.showExitEditDialog,
      handleSaveAndExit: () => dialogManager.handleSaveAndExit(saveNote, exitEditMode),
      handleDiscardAndExit: () => dialogManager.handleDiscardAndExit(exitEditMode),
      openCreateDialog: dialogManager.openCreateDialog,
      closeCreateDialog: dialogManager.closeCreateDialog,
      openRenameDialog: dialogManager.openRenameDialog,
      closeRenameDialog: dialogManager.closeRenameDialog,
      openDeleteDialog: dialogManager.openDeleteDialog,
      closeDeleteDialog: dialogManager.closeDeleteDialog,
      openSettingsPane: () => configService.openPane(() => focusManager.focusSearch()),
      closeSettingsPane: () => configService.closePane(() => focusManager.focusSearch()),
      handleDeleteKeyPress: () => dialogManager.handleDeleteKeyPress(deleteNote),
      clearHighlights: contentManager.clearHighlights,
      clearSearch: searchManager.clearSearch,
      invoke,
    };
  },

  // Application initialization
  async initialize(): Promise<() => void> {
    await tick();

    const configExists = await invoke<boolean>("config_exists");
    if (!configExists) {
      await configService.openPane(() => focusManager.focusSearch());
    } else {
      focusManager.focusSearch();
      const notes = await searchManager.searchImmediate('');
      state.filteredNotes = notes;
    }

    const unlisten = await listen("open-preferences", async () => {
      await configService.openPane(() => focusManager.focusSearch());
    });

    return () => {
      searchManager.abort();
      if (contentRequestController) contentRequestController.abort();
      unlisten();
    };
  }
};
