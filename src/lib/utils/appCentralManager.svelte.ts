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
  selectedIndex: number;
}

const state = $state<CentralAppState>({
  selectedIndex: -1,
});

const isLoading = $derived(searchManager.isLoading);
const areHighlightsCleared = $derived(contentManager.areHighlightsCleared);
const filteredNotes = $derived(searchManager.filteredNotes);
const query = $derived(searchManager.searchInput);

const selectedNote = $derived.by(() => {
  const notes = filteredNotes;
  let index = state.selectedIndex;

  if (notes.length === 0) {
    return null;
  }

  if (index === -1 || index >= notes.length) {
    index = 0;
  }

  return notes[index] || null;
});


let contentRequestController: AbortController | null = null;

function setupReactiveEffects() {
  $effect(() => {
    const notes = filteredNotes;
    const currentIndex = state.selectedIndex;

    // Exit edit mode when selection gets normalized (e.g., when filtered notes change)
    if (notes.length > 0 && (currentIndex === -1 || currentIndex >= notes.length)) {
      editorManager.exitEditMode();
    }
  });

  $effect(() => {
    const notes = filteredNotes;
    let index = state.selectedIndex;

    // Normalize for scrolling purposes only
    if (notes.length > 0) {
      if (index === -1 || index >= notes.length) {
        index = 0;
      }
      requestAnimationFrame(() => {
        focusManager.scrollToSelected(index);
      });
    }
  });

  $effect(() => {
    const note = selectedNote;

    if (!note) {
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
        const content = await noteService.getContent(note);

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

  // Update content highlighting reactively
  $effect(() => {
    contentManager.updateHighlighterState({
      query: query,
      areHighlightsCleared: areHighlightsCleared
    });
  });
}

// State setters
function setSelectedIndex(value: number): void {
  state.selectedIndex = value;
}

// Business logic functions
async function deleteNote(): Promise<void> {
  if (!selectedNote) return;

  const result = await noteService.delete(selectedNote);

  if (result.success) {
    // Refresh the notes list
    await searchManager.searchImmediate(searchManager.searchInput);

    // Close dialog and focus search
    dialogManager.closeDeleteDialog();
    await tick();
    focusManager.focusSearch();
  }
}

async function createNote(noteNameParam?: string): Promise<void> {
  const inputNoteName = noteNameParam || dialogManager.newNoteName.trim();
  if (!inputNoteName.trim()) return;

  const result = await noteService.create(inputNoteName);

  if (result.success) {
    // Refresh the notes list
    await searchManager.searchImmediate('');

    // Find and select the new note
    const noteIndex = searchManager.filteredNotes.findIndex(note => note === result.noteName);
    if (noteIndex >= 0) {
      state.selectedIndex = noteIndex;
    }

    // Close dialog and focus search
    dialogManager.closeCreateDialog();
    await tick();
    focusManager.focusSearch();

    enterEditMode();
  }
}

async function renameNote(newNameParam?: string): Promise<void> {
  const inputNewName = newNameParam || dialogManager.newNoteNameForRename.trim();
  if (!inputNewName.trim() || !selectedNote) return;

  const result = await noteService.rename(selectedNote, inputNewName);

  if (result.success) {
    // Refresh the notes list
    await searchManager.searchImmediate(searchManager.searchInput);

    // Select the renamed note
    const noteIndex = searchManager.filteredNotes.findIndex(note => note === result.newName);
    if (noteIndex >= 0) {
      state.selectedIndex = noteIndex;
    }

    // Close dialog
    dialogManager.closeRenameDialog();
  }
}

async function saveNote(): Promise<void> {
  if (!selectedNote) return;

  const result = await editorManager.saveNote(selectedNote);

  if (result.success) {
    try {
      const refreshResult = await contentManager.refreshAfterSave(selectedNote, searchManager.searchInput);
      searchManager.updateState({ filteredNotes: refreshResult.searchResults });
    } catch (e) {
      console.error("Failed to refresh after save:", e);
    }
  } else {
    console.error("Failed to save note:", result.error);
  }
}

async function saveAndExitNote(): Promise<void> {
  await saveNote();
  exitEditMode();
}

function selectNote(note: string, index: number): void {
  if (state.selectedIndex !== index) {
    state.selectedIndex = index;
  }
}

async function enterEditMode(): Promise<void> {
  if (selectedNote) {
    await editorManager.enterEditMode(
      selectedNote,
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
  get query(): string {
    return query;
  },

  get isLoading(): boolean {
    return isLoading;
  },

  get areHighlightsCleared(): boolean {
    return areHighlightsCleared;
  },

  get filteredNotes(): string[] {
    return filteredNotes;
  },

  get selectedNote(): string | null {
    return selectedNote;
  },

  get selectedIndex(): number {
    return state.selectedIndex;
  },

  // State setters

  updateFilteredNotes(notes: string[]): void {
    searchManager.updateState({ filteredNotes: notes });
  },

  resetState(): void {
    searchManager.searchInput = '';
    state.selectedIndex = -1;
    searchManager.updateState({
      filteredNotes: [],
      isLoading: false
    });
    contentManager.areHighlightsCleared = false;
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
  saveAndExitNote,
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
      filteredNotes: filteredNotes,
      selectedNote: selectedNote,
      noteContentElement: focusManager.noteContentElement,
      areHighlightsCleared: areHighlightsCleared,
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
      saveAndExitNote,
      invoke,
      showDeleteDialog: () => dialogManager.openDeleteDialog(),
      showCreateDialog: () => dialogManager.openCreateDialog(query, contentManager.highlightedContent),
      showRenameDialog: () => dialogManager.openRenameDialog(selectedNote ?? undefined),
      openSettingsPane: async () => {
        await configService.openPane();
        focusManager.focusSearch();
      },
      clearHighlights: contentManager.clearHighlights,
      clearSearch: searchManager.clearSearch,
    };
  },

  // Context provider
  get context() {
    return {
      state: {
        get query() { return query; },
        get isLoading() { return isLoading; },
        get areHighlightsCleared() { return areHighlightsCleared; },
        get filteredNotes() { return filteredNotes; },
        get selectedNote() { return selectedNote; },
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
      saveAndExitNote,
      enterEditMode,
      exitEditMode,
      showExitEditDialog: dialogManager.showExitEditDialog,
      handleSaveAndExit: () => dialogManager.handleSaveAndExit(saveAndExitNote),
      handleDiscardAndExit: () => dialogManager.handleDiscardAndExit(exitEditMode),
      openCreateDialog: () => dialogManager.openCreateDialog(query, contentManager.highlightedContent),
      closeCreateDialog: dialogManager.closeCreateDialog,
      openRenameDialog: () => dialogManager.openRenameDialog(selectedNote ?? undefined),
      closeRenameDialog: dialogManager.closeRenameDialog,
      openDeleteDialog: dialogManager.openDeleteDialog,
      closeDeleteDialog: dialogManager.closeDeleteDialog,
      openSettingsPane: async () => {
        await configService.openPane();
        focusManager.focusSearch();
      },
      closeSettingsPane: () => {
        configService.closePane();
        focusManager.focusSearch();
      },
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
      await configService.openPane();
      focusManager.focusSearch();
    } else {
      focusManager.focusSearch();
      await searchManager.searchImmediate('');
    }

    const unlisten = await listen("open-preferences", async () => {
      await configService.openPane();
      focusManager.focusSearch();
    });

    return () => {
      searchManager.abort();
      if (contentRequestController) contentRequestController.abort();
      unlisten();
    };
  }
};
