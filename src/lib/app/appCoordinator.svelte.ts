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

export function createAppCoordinator(deps: AppCoordinatorDeps) {
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

  function setupReactiveEffects() {
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

    get context() {
      return {
        state: {
          get query() { return query; },
          get isLoading() { return isLoading; },
          get areHighlightsCleared() { return areHighlightsCleared; },
          get filteredNotes() { return filteredNotes; },
          get selectedNote() { return selectedNote; },
          get selectedIndex() { return selectedIndex; },
        },
        dialogManager,
        editorManager,
        focusManager,
        contentManager,
        selectNote,
        deleteNote: () => noteActions.deleteNote(selectedNote),
        createNote: noteActions.createNote,
        renameNote: (newName?: string) => noteActions.renameNote(selectedNote, newName),
        saveNote: () => noteActions.saveNote(selectedNote),
        saveAndExitNote,
        enterEditMode: () => noteActions.enterEditMode(selectedNote!),
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
        openSettingsPane: settingsActions.openSettingsPane,
        closeSettingsPane: settingsActions.closeSettingsPane,
        handleDeleteKeyPress: () => dialogManager.handleDeleteKeyPress(() => noteActions.deleteNote(selectedNote)),
        clearHighlights: contentManager.clearHighlights,
        clearSearch: searchManager.clearSearch,
        invoke,
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
        if (contentRequestController) contentRequestController.abort();
        cleanupEffects();
        unlisten();
      };
    }
  };
}
