<script lang="ts">
import { onMount, setContext } from "svelte";
import AppLayout from "../lib/ui/AppLayout.svelte";
import SearchInput from "../lib/ui/SearchInput.svelte";
import NoteList from "../lib/ui/NoteList.svelte";
import NoteView from "../lib/ui/NoteView.svelte";
import ConfirmationDialog from "../lib/ui/ConfirmationDialog.svelte";
import InputDialog from "../lib/ui/InputDialog.svelte";
import DeleteDialog from "../lib/ui/DeleteDialog.svelte";
import SettingsPane from "../lib/ui/SettingsPane.svelte";
import DebugPanel from "../lib/ui/DebugPanel.svelte";
// Keyboard handler now integrated in appCoordinator
import { createAppCoordinator } from '../lib/app/appCoordinator.svelte';
import { createSearchManager } from '../lib/core/searchManager.svelte';
import { createEditorManager } from '../lib/core/editorManager.svelte';
import { createFocusManager } from '../lib/core/focusManager.svelte';
import { configService } from '../lib/services/configService.svelte';

// Create all managers using factories
const searchManager = createSearchManager();
const editorManager = createEditorManager();
const focusManager = createFocusManager();

// Create the coordinator with dependencies
const appCoordinator = createAppCoordinator({
  searchManager,
  editorManager,
  focusManager
});

const context = appCoordinator.context;

// Set context for child components
setContext<{
  searchManager: typeof searchManager;
  editorManager: typeof editorManager;
  focusManager: typeof focusManager;
  appCoordinator: typeof appCoordinator;
}>('managers', {
  searchManager,
  editorManager,
  focusManager,
  appCoordinator
});

const handleKeydown = appCoordinator.keyboardActions;

appCoordinator.setupReactiveEffects();

onMount(() => {
  (async () => {
    const cleanup = await appCoordinator.initialize();
    return cleanup;
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
      onClose={() => {
        configService.closePane();
        focusManager.focusSearch();
      }}
      onRefresh={(notes) => {
        appCoordinator.updateFilteredNotes(notes);
      }}
    />

    <DeleteDialog
      show={context.dialogManager.showDeleteDialog}
      noteName={appCoordinator.selectedNote || ''}
      deleteKeyPressCount={context.dialogManager.deleteKeyPressCount}
      onConfirm={appCoordinator.deleteNote}
      onCancel={context.dialogManager.closeDeleteDialog}
      onKeyPress={() => context.dialogManager.handleDeleteKeyPress(() => appCoordinator.deleteNote())}
    />

    <InputDialog
      show={context.dialogManager.showCreateDialog}
      title="Create New Note"
      value={context.dialogManager.newNoteName}
      placeholder="Enter note name (extension will be .md)"
      confirmText="Create"
      cancelText="Cancel"
      onConfirm={(value) => appCoordinator.createNote(value)}
      onCancel={context.dialogManager.closeCreateDialog}
      onInput={(value) => context.dialogManager.setNewNoteName(value)}
    />

    <InputDialog
      show={context.dialogManager.showRenameDialog}
      title="Rename Note"
      value={context.dialogManager.newNoteNameForRename}
      placeholder="Enter new note name"
      confirmText="Rename"
      cancelText="Cancel"
      autoSelect={true}
      onConfirm={(value) => appCoordinator.renameNote(value)}
      onCancel={context.dialogManager.closeRenameDialog}
      onInput={(value) => context.dialogManager.setNewNoteNameForRename(value)}
    />

    <ConfirmationDialog
      show={context.dialogManager.showUnsavedChangesDialog}
      title="Unsaved Changes"
      message="You have unsaved changes. What would you like to do?"
      confirmText="Save and Exit"
      cancelText="Discard Changes"
      variant="default"
      onConfirm={() => context.dialogManager.handleSaveAndExit(appCoordinator.saveAndExitNote)}
      onCancel={() => context.dialogManager.handleDiscardAndExit(appCoordinator.exitEditMode)}
    />
  </div>
</AppLayout>

<DebugPanel />

