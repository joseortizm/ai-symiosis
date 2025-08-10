<script lang="ts">
import { onMount } from "svelte";
import AppLayout from "../lib/components/AppLayout.svelte";
import SearchInput from "../lib/components/SearchInput.svelte";
import NoteList from "../lib/components/NoteList.svelte";
import NoteView from "../lib/components/NoteView.svelte";
import ConfirmationDialog from "../lib/components/ConfirmationDialog.svelte";
import InputDialog from "../lib/components/InputDialog.svelte";
import DeleteDialog from "../lib/components/DeleteDialog.svelte";
import SettingsPane from "../lib/components/SettingsPane.svelte";
import DebugPanel from "../lib/components/DebugPanel.svelte";
import { createKeyboardHandler } from '../lib/keyboardHandler';
import { appCentralManager } from '../lib/utils/appCentralManager.svelte';
import { configService } from '../lib/services/configService.svelte';
import { focusManager } from '../lib/utils/focusManager.svelte';

const context = appCentralManager.context;

const handleKeydown = createKeyboardHandler(
  () => appCentralManager.keyboardState,
  appCentralManager.keyboardActions
);

appCentralManager.setupReactiveEffects();

onMount(() => {
  (async () => {
    const cleanup = await appCentralManager.initialize();
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
        appCentralManager.updateFilteredNotes(notes);
      }}
    />

    <DeleteDialog
      show={context.dialogManager.showDeleteDialog}
      noteName={appCentralManager.selectedNote || ''}
      deleteKeyPressCount={context.dialogManager.deleteKeyPressCount}
      onConfirm={appCentralManager.deleteNote}
      onCancel={context.dialogManager.closeDeleteDialog}
      onKeyPress={() => context.dialogManager.handleDeleteKeyPress(() => appCentralManager.deleteNote())}
    />

    <InputDialog
      show={context.dialogManager.showCreateDialog}
      title="Create New Note"
      value={context.dialogManager.newNoteName}
      placeholder="Enter note name (extension will be .md)"
      confirmText="Create"
      cancelText="Cancel"
      onConfirm={(value) => appCentralManager.createNote(value)}
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
      onConfirm={(value) => appCentralManager.renameNote(value)}
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
      onConfirm={() => context.dialogManager.handleSaveAndExit(appCentralManager.saveAndExitNote)}
      onCancel={() => context.dialogManager.handleDiscardAndExit(appCentralManager.exitEditMode)}
    />
  </div>
</AppLayout>

<DebugPanel />
