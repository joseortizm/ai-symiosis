/**
 * App Layer - Application Coordinator
 * Central coordinator for app-wide state, actions, and effects.
 * Maintains separation of concerns across the application architecture.
 */

import { invoke } from "@tauri-apps/api/core";
import { tick } from "svelte";
import { listen } from "@tauri-apps/api/event";
import { createDialogManager } from '../core/dialogManager.svelte';
import { createContentManager } from '../core/contentManager.svelte';
import { noteService } from '../services/noteService.svelte';
import { configService } from '../services/configService.svelte';
import { createNoteActions } from './actions/note.svelte';
import { createSearchActions } from './actions/search.svelte';
import { createSettingsActions } from './actions/settings.svelte';
import { createKeyboardActions } from './actions/keyboard.svelte';
import { setupAppEffects } from './effects/app.svelte';


interface AppCoordinatorDeps {
  searchManager: ReturnType<typeof import('../core/searchManager.svelte').createSearchManager>;
  editorManager: ReturnType<typeof import('../core/editorManager.svelte').createEditorManager>;
  focusManager: ReturnType<typeof import('../core/focusManager.svelte').createFocusManager>;
}

export interface AppCoordinator {
  readonly query: string;
  readonly isLoading: boolean;
  readonly areHighlightsCleared: boolean;
  readonly filteredNotes: string[];
  readonly selectedNote: string | null;
  readonly selectedIndex: number;
  readonly keyboardState: {
    isSearchInputFocused: boolean;
    isEditMode: boolean;
    isNoteContentFocused: boolean;
    selectedIndex: number;
    filteredNotes: string[];
    selectedNote: string | null;
    noteContentElement: HTMLElement | null;
    areHighlightsCleared: boolean;
    isEditorDirty: boolean;
    query: string;
  };
  readonly keyboardActions: (event: KeyboardEvent) => Promise<void>;
  readonly managers: any;
  readonly state: any;
  readonly actions: any;
  setupReactiveEffects(): () => void;
  updateFilteredNotes(notes: string[]): void;
  resetState(): void;
  setSelectedIndex(index: number): void;
  deleteNote(): Promise<void>;
  createNote(noteName?: string): Promise<void>;
  renameNote(newName?: string): Promise<void>;
  saveNote(): Promise<void>;
  saveAndExitNote(): Promise<void>;
  selectNote(note: string, index: number): void;
  enterEditMode(): Promise<void>;
  exitEditMode(): void;
  initialize(): Promise<() => void>;
}

export function createAppCoordinator(deps: AppCoordinatorDeps): AppCoordinator {
  const { searchManager, editorManager, focusManager } = deps;

  let selectedIndex = $state(-1);

  const dialogManager = createDialogManager({
    focusSearch: () => focusManager.focusSearch()
  });

  const contentManager = createContentManager({
    noteService,
    searchManager,
    getNoteContentElement: () => focusManager.noteContentElement,
    refreshSearch: (query: string) => searchManager.refreshSearch(query),
    invoke
  });

  const isLoading = $derived(searchManager.isLoading);
  const areHighlightsCleared = $derived(searchManager.areHighlightsCleared);
  const filteredNotes = $derived(searchManager.filteredNotes);
  const query = $derived(searchManager.searchInput);

  const selectedNote = $derived.by(() => {
    const notes = filteredNotes;
    let index = selectedIndex;

    if (notes.length === 0) {
      return null;
    }

    if (index === -1 || index >= notes.length) {
      index = 0;
    }

    return notes[index] || null;
  });

  let contentRequestController: AbortController | null = null;

  function setSelectedIndex(value: number): void {
    selectedIndex = value;
  }

  const noteActions = createNoteActions({
    noteService,
    searchManager,
    dialogManager,
    focusManager,
    editorManager,
    contentManager,
    setSelectedIndex
  });

  const searchActions = createSearchActions({
    searchManager,
    contentManager
  });

  const settingsActions = createSettingsActions({
    configService,
    focusManager
  });

  function exitEditMode(): void {
    editorManager.exitEditMode();
    focusManager.focusSearch();
  }

  async function saveAndExitNote(): Promise<void> {
    await noteActions.saveNote(selectedNote);
    exitEditMode();
  }

  const keyboardActions = createKeyboardActions({
    setSelectedIndex,
    enterEditMode: () => noteActions.enterEditMode(selectedNote!),
    exitEditMode,
    saveAndExitNote,
    showExitEditDialog: dialogManager.showExitEditDialog,
    showDeleteDialog: () => dialogManager.openDeleteDialog(),
    showCreateDialog: () => dialogManager.openCreateDialog(query, contentManager.highlightedContent),
    showRenameDialog: () => dialogManager.openRenameDialog(selectedNote ?? undefined),
    openSettingsPane: settingsActions.openSettingsPane,
    clearHighlights: contentManager.clearHighlights,
    clearSearch: searchManager.clearSearch,
    focusSearch: () => focusManager.focusSearch(),
  });

  function setupReactiveEffects(): () => void {
    return setupAppEffects({
      getFilteredNotes: () => filteredNotes,
      getSelectedIndex: () => selectedIndex,
      getSelectedNote: () => selectedNote,
      getAreHighlightsCleared: () => areHighlightsCleared,
      setSelectedIndex,
      editorManager,
      focusManager,
      contentManager,
      noteService,
      contentRequestController: {
        current: contentRequestController,
        set: (controller) => { contentRequestController = controller; }
      }
    });
  }

  function resetState(): void {
    searchManager.searchInput = '';
    selectedIndex = -1;
    searchManager.setFilteredNotes([]);
    searchManager.areHighlightsCleared = false;
    if (contentRequestController) {
      contentRequestController.abort();
      contentRequestController = null;
    }
  }

  function selectNote(note: string, index: number): void {
    if (selectedIndex !== index) {
      selectedIndex = index;
    }
  }

  return {
    setupReactiveEffects,

    get query(): string { return query; },
    get isLoading(): boolean { return isLoading; },
    get areHighlightsCleared(): boolean { return areHighlightsCleared; },
    get filteredNotes(): string[] { return filteredNotes; },
    get selectedNote(): string | null { return selectedNote; },
    get selectedIndex(): number { return selectedIndex; },

    updateFilteredNotes: searchActions.updateFilteredNotes,
    resetState,
    setSelectedIndex,

    deleteNote: () => noteActions.deleteNote(selectedNote),
    createNote: noteActions.createNote,
    renameNote: (newName?: string) => noteActions.renameNote(selectedNote, newName),
    saveNote: () => noteActions.saveNote(selectedNote),
    saveAndExitNote,
    selectNote,
    enterEditMode: () => noteActions.enterEditMode(selectedNote!),
    exitEditMode,

    get keyboardState() {
      return {
        isSearchInputFocused: focusManager.isSearchInputFocused,
        isEditMode: editorManager.isEditMode,
        isNoteContentFocused: focusManager.isNoteContentFocused,
        selectedIndex: selectedIndex,
        filteredNotes: filteredNotes,
        selectedNote: selectedNote,
        noteContentElement: focusManager.noteContentElement,
        areHighlightsCleared: areHighlightsCleared,
        isEditorDirty: editorManager.isDirty,
        query: query,
      };
    },

    get keyboardActions() {
      return keyboardActions.createKeyboardHandler(() => ({
        isSearchInputFocused: focusManager.isSearchInputFocused,
        isEditMode: editorManager.isEditMode,
        isNoteContentFocused: focusManager.isNoteContentFocused,
        selectedIndex: selectedIndex,
        filteredNotes: filteredNotes,
        selectedNote: selectedNote,
        noteContentElement: focusManager.noteContentElement,
        areHighlightsCleared: areHighlightsCleared,
        isEditorDirty: editorManager.isDirty,
        query: query,
      }));
    },

    get managers() {
      return {
        searchManager,
        editorManager,
        focusManager,
        contentManager,
        dialogManager,
      };
    },

    get state() {
      return {
        get query() { return query; },
        get isLoading() { return isLoading; },
        get areHighlightsCleared() { return areHighlightsCleared; },
        get filteredNotes() { return filteredNotes; },
        get selectedNote() { return selectedNote; },
        get selectedIndex() { return selectedIndex; },
      };
    },

    get actions() {
      return {
        selectNote,
        deleteNote: () => noteActions.deleteNote(selectedNote),
        createNote: noteActions.createNote,
        renameNote: (newName?: string) => noteActions.renameNote(selectedNote, newName),
        saveNote: () => noteActions.saveNote(selectedNote),
        saveAndExitNote,
        enterEditMode: () => noteActions.enterEditMode(selectedNote!),
        exitEditMode,
      };
    },


    async initialize(): Promise<() => void> {
      await tick();

      const configExists = await invoke<boolean>("config_exists");
      if (!configExists) {
        await settingsActions.openSettingsPane();
      } else {
        focusManager.focusSearch();
        await searchManager.searchImmediate('');
      }

      const unlisten = await listen("open-preferences", async () => {
        await settingsActions.openSettingsPane();
      });

      const cleanupEffects = setupReactiveEffects();

      return () => {
        searchManager.abort();
        if (contentRequestController) {
          contentRequestController.abort();
          contentRequestController = null;
        }
        cleanupEffects();
        unlisten();
      };
    }
  };
}
