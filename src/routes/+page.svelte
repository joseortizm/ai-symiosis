<!--
Route Layer - Main Page
Root application component that initializes managers, context, and handles app lifecycle.
Composes all UI components and provides keyboard event handling for the entire app.
-->

<script lang="ts">
  import { onMount, setContext } from 'svelte'
  import AppLayout from '../lib/ui/AppLayout.svelte'
  import SearchInput from '../lib/ui/SearchInput.svelte'
  import NoteList from '../lib/ui/NoteList.svelte'
  import NoteView from '../lib/ui/NoteView.svelte'
  import ConfirmationDialog from '../lib/ui/ConfirmationDialog.svelte'
  import InputDialog from '../lib/ui/InputDialog.svelte'
  import DeleteDialog from '../lib/ui/DeleteDialog.svelte'
  import SettingsPane from '../lib/ui/SettingsPane.svelte'
  import ProgressOverlay from '../lib/ui/ProgressOverlay.svelte'
  import DebugPanel from '../lib/ui/DebugPanel.svelte'
  import { createAppCoordinator } from '../lib/app/appCoordinator.svelte'
  import { createSearchManager } from '../lib/core/searchManager.svelte'
  import { createEditorManager } from '../lib/core/editorManager.svelte'
  import { createFocusManager } from '../lib/core/focusManager.svelte'
  import { configService } from '../lib/services/configService.svelte'
  import { noteService } from '../lib/services/noteService.svelte'

  // Create all managers using factories
  const searchManager = createSearchManager({ noteService })
  const editorManager = createEditorManager({ noteService })
  const focusManager = createFocusManager()

  // Create the coordinator with dependencies
  const appCoordinator = createAppCoordinator({
    searchManager,
    editorManager,
    focusManager,
  })

  // Set context for child components
  setContext('managers', {
    ...appCoordinator.managers,
    appCoordinator,
  })

  setContext('state', appCoordinator.state)
  setContext('actions', appCoordinator.actions)

  // Access properties directly since this is the root component
  const { dialogManager, progressManager } = appCoordinator.managers
  const appState = appCoordinator.state
  const actions = appCoordinator.actions

  const handleKeydown = appCoordinator.keyboardActions

  appCoordinator.setupReactiveEffects()

  onMount(() => {
    let cleanup: (() => void) | undefined
    ;(async () => {
      cleanup = await appCoordinator.initialize()
    })()
    return () => cleanup?.()
  })
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
        configService.closePane()
        focusManager.focusSearch()
      }}
    />

    <DeleteDialog
      show={dialogManager.showDeleteDialog}
      noteName={appState.selectedNote || ''}
      deleteKeyPressCount={dialogManager.deleteKeyPressCount}
      onConfirm={actions.deleteNote}
      onCancel={dialogManager.closeDeleteDialog}
      onKeyPress={() =>
        dialogManager.handleDeleteKeyPress(() => actions.deleteNote())}
    />

    <InputDialog
      show={dialogManager.showCreateDialog}
      title="Create New Note"
      value={dialogManager.newNoteName}
      placeholder="Enter note name (extension will be .md)"
      confirmText="Create"
      cancelText="Cancel"
      onConfirm={(value) => actions.createNote(value)}
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
      onConfirm={(value) => actions.renameNote(value)}
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
      onConfirm={() => dialogManager.handleSaveAndExit(actions.saveAndExitNote)}
      onCancel={() => dialogManager.handleDiscardAndExit(actions.exitEditMode)}
    />

    <ProgressOverlay
      show={progressManager.isLoading}
      message={progressManager.message}
      error={progressManager.error}
    />
  </div>
</AppLayout>

<DebugPanel />
