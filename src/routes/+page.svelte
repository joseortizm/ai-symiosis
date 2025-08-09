<script lang="ts">
import { invoke } from "@tauri-apps/api/core";
import { onMount, tick } from "svelte";
import { listen } from "@tauri-apps/api/event";
import AppLayout from "../lib/components/AppLayout.svelte";
import SearchInput from "../lib/components/SearchInput.svelte";
import NoteList from "../lib/components/NoteList.svelte";
import NoteView from "../lib/components/NoteView.svelte";
import ConfirmationDialog from "../lib/components/ConfirmationDialog.svelte";
import InputDialog from "../lib/components/InputDialog.svelte";
import DeleteDialog from "../lib/components/DeleteDialog.svelte";
import SettingsPane from "../lib/components/SettingsPane.svelte";
import { createKeyboardHandler } from '../lib/keyboardHandler';
import { setAppContext } from '../lib/context/app.svelte';
import { searchManager } from '../lib/utils/searchManager.svelte';
import { dialogManager } from '../lib/utils/dialogManager.svelte';
import { noteService } from '../lib/services/noteService.svelte';
import { configService } from '../lib/services/configService.svelte';
import { editorManager } from '../lib/utils/editorManager.svelte';
import { focusManager } from '../lib/utils/focusManager.svelte';
import { contentManager } from '../lib/utils/contentManager.svelte';

interface SearchNotesResponse {
  [key: string]: string[];
}

type TauriInvokeResponse<T> = Promise<T>;

const appState = $state({
  searchInput: '',
  query: '',
  isLoading: false,
  areHighlightsCleared: false,

  filteredNotes: [] as string[],
  selectedNote: null as string | null,
  selectedIndex: -1,
});

let contentRequestController: AbortController | null = null;

async function deleteNote(): Promise<void> {
  if (!appState.selectedNote) return;

  await noteService.delete(
    appState.selectedNote,
    searchManager,
    dialogManager,
    (notes) => { appState.filteredNotes = notes; },
    appState.searchInput,
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
      appState.filteredNotes = notes;
      // Select the new note
      const noteIndex = notes.findIndex(note => note === (inputNoteName.includes('.') ? inputNoteName : `${inputNoteName}.md`));
      if (noteIndex >= 0) {
        appState.selectedIndex = noteIndex;
      }
    },
    () => focusManager.focusSearch()
  );
}

async function renameNote(newNameParam?: string): Promise<void> {
  const inputNewName = newNameParam || dialogManager.newNoteNameForRename.trim();
  if (!inputNewName.trim() || !appState.selectedNote) return;

  await noteService.rename(
    appState.selectedNote,
    inputNewName,
    searchManager,
    dialogManager,
    (notes) => { appState.filteredNotes = notes; },
    (noteName) => {
      const noteIndex = appState.filteredNotes.findIndex(note => note === noteName);
      if (noteIndex >= 0) {
        appState.selectedIndex = noteIndex;
      }
    },
    appState.searchInput
  );
}


$effect(() => {
  searchManager.updateSearchInputWithEffects(
    appState.searchInput,
    (query) => {
      appState.query = query;
    },
    (cleared) => {
      appState.areHighlightsCleared = cleared;
    }
  );
});

$effect(() => {
  appState.isLoading = searchManager.isLoading;
});

function arraysEqual(a: string[], b: string[]): boolean {
  return a.length === b.length && a.every((val, i) => val === b[i]);
}

$effect(() => {
  const notes = searchManager.filteredNotes;
  if (!arraysEqual(notes, appState.filteredNotes)) {
    appState.filteredNotes = notes;
    if (notes.length === 0) {
      appState.selectedIndex = -1;
    } else {
      appState.selectedIndex = 0;
    }
  }
});

$effect(() => {
  const newSelectedNote = appState.filteredNotes.length > 0 && appState.selectedIndex !== -1
    ? appState.filteredNotes[appState.selectedIndex]
    : null;
  if (newSelectedNote !== appState.selectedNote) {
    appState.selectedNote = newSelectedNote;
    editorManager.exitEditMode();
  }
});

$effect(() => {
  if (appState.selectedIndex >= 0) {
    requestAnimationFrame(() => {
      focusManager.scrollToSelected(appState.selectedIndex);
    });
  }
});

$effect(() => {
  if (!appState.selectedNote) {
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
      const content = await contentManager.getNoteContent(appState.selectedNote!);

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
    query: appState.query,
    areHighlightsCleared: appState.areHighlightsCleared
  });
});

$effect(() => {
  dialogManager.updateState({
    selectedNote: appState.selectedNote,
    query: appState.query,
    highlightedContent: contentManager.highlightedContent,
    searchElement: focusManager.searchElement
  });
});

function selectNote(note: string, index: number): void {
  if (appState.selectedIndex !== index) {
    appState.selectedIndex = index;
  }
}

async function enterEditMode(): Promise<void> {
  if (appState.selectedNote) {
    await editorManager.enterEditMode(
      appState.selectedNote,
      contentManager.noteContent,
      focusManager.noteContentElement || undefined
    );
  }
}

function exitEditMode(): void {
  editorManager.exitEditMode();
  focusManager.focusSearch();
}

async function saveNote(): Promise<void> {
  if (!appState.selectedNote) return;

  const result = await editorManager.saveAndExit(appState.selectedNote);

  if (result.success) {
    try {
      const refreshResult = await contentManager.refreshAfterSave(appState.selectedNote, appState.searchInput);
      appState.filteredNotes = refreshResult.searchResults;

      await tick();
      focusManager.focusSearch();
    } catch (e) {
      console.error("Failed to refresh after save:", e);
    }
  } else {
    console.error("Failed to save note:", result.error);
  }
}

const handleKeydown = createKeyboardHandler(
  () => ({
    isSearchInputFocused: focusManager.isSearchInputFocused,
    isEditMode: editorManager.isEditMode,
    isNoteContentFocused: focusManager.isNoteContentFocused,
    selectedIndex: appState.selectedIndex,
    filteredNotes: appState.filteredNotes,
    selectedNote: appState.selectedNote,
    noteContentElement: focusManager.noteContentElement,
    searchElement: focusManager.searchElement,
    query: appState.query,
    areHighlightsCleared: appState.areHighlightsCleared,
    isEditorDirty: editorManager.isDirty,
  }),
  {
    setSelectedIndex: (value: number) => appState.selectedIndex = value,
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
  }
);

setAppContext({
  state: appState,
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
  handleDeleteKeyPress: () => dialogManager.handleDeleteKeyPress(() => deleteNote()),
  clearHighlights: contentManager.clearHighlights,
  clearSearch: searchManager.clearSearch,
  invoke,
});

onMount(() => {
  (async () => {
    await tick();

    const configExists = await invoke<boolean>("config_exists");
    if (!configExists) {
      await configService.openPane(() => focusManager.focusSearch());
    } else {
      focusManager.focusSearch();
      const notes = await searchManager.searchImmediate('');
      appState.filteredNotes = notes;
    }

    const unlisten = await listen("open-preferences", async () => {
      await configService.openPane(() => focusManager.focusSearch());
    });

    return () => {
      searchManager.abort();
      if (contentRequestController) contentRequestController.abort();
      unlisten();
    };
  })();
});
</script>

<svelte:window onkeydown={handleKeydown} />

<AppLayout>
  <SearchInput slot="search" />
  <NoteList slot="list" />
  <NoteView slot="view" />

  <div slot="modals">
    <SettingsPane
      show={configService.isVisible}
      onClose={() => configService.closePane(() => focusManager.focusSearch())}
      onRefresh={(notes) => {
        appState.filteredNotes = notes;
      }}
    />

    <DeleteDialog
      show={dialogManager.showDeleteDialog}
      noteName={appState.selectedNote || ''}
      deleteKeyPressCount={dialogManager.deleteKeyPressCount}
      onConfirm={deleteNote}
      onCancel={dialogManager.closeDeleteDialog}
      onKeyPress={() => dialogManager.handleDeleteKeyPress(() => deleteNote())}
    />

    <InputDialog
      show={dialogManager.showCreateDialog}
      title="Create New Note"
      value={dialogManager.newNoteName}
      placeholder="Enter note name (extension will be .md)"
      confirmText="Create"
      cancelText="Cancel"
      onConfirm={(value) => createNote(value)}
      onCancel={dialogManager.closeCreateDialog}
      onInput={(value) => dialogManager.setNewNoteName(value)}
    />

    <InputDialog
      show={dialogManager.showRenameDialog}
      title="Rename Note"
      value={dialogManager.newNoteNameForRename}
      placeholder="Enter new note name"
      confirmText="Rename"
      cancelText="Cancel"
      autoSelect={true}
      onConfirm={(value) => renameNote(value)}
      onCancel={dialogManager.closeRenameDialog}
      onInput={(value) => dialogManager.setNewNoteNameForRename(value)}
    />

    <ConfirmationDialog
      show={dialogManager.showUnsavedChangesDialog}
      title="Unsaved Changes"
      message="You have unsaved changes. What would you like to do?"
      confirmText="Save and Exit"
      cancelText="Discard Changes"
      variant="default"
      onConfirm={() => dialogManager.handleSaveAndExit(saveNote, exitEditMode)}
      onCancel={() => dialogManager.handleDiscardAndExit(exitEditMode)}
    />
  </div>
</AppLayout>

<style>

</style>

