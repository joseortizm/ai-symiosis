/**
 * Core Layer - Dialog Manager
 * Modal dialog states for create, delete, rename, and exit confirmation dialogs.
 * Handles dialog input values, visibility state, and focus management after dialog actions.
 */

export interface DialogManagerDeps {
  focusSearch: () => void;
}

interface DialogContext {
  selectedNote: string | null;
  query: string;
  highlightedContent: string;
}

export interface DialogManager {
  readonly showCreateDialog: boolean;
  readonly showRenameDialog: boolean;
  readonly showDeleteDialog: boolean;
  readonly showUnsavedChangesDialog: boolean;
  readonly newNoteName: string;
  readonly newNoteNameForRename: string;
  readonly deleteKeyPressCount: number;
  openCreateDialog(query?: string, highlightedContent?: string): void;
  closeCreateDialog(): void;
  openRenameDialog(selectedNote?: string): void;
  closeRenameDialog(): void;
  openDeleteDialog(): void;
  closeDeleteDialog(): void;
  openUnsavedChangesDialog(): void;
  closeUnsavedChangesDialog(): void;
  showExitEditDialog(): void;
  handleSaveAndExit(saveAndExitNote: () => Promise<void>): Promise<void>;
  handleDiscardAndExit(exitEditMode: () => void): void;
  handleDeleteKeyPress(onConfirmDelete: () => Promise<void>): void;
  setNewNoteName(value: string): void;
  setNewNoteNameForRename(value: string): void;
}

export function createDialogManager(deps: DialogManagerDeps): DialogManager {
  const context = $state<DialogContext>({
    selectedNote: null,
    query: '',
    highlightedContent: ''
  });

  let showCreateDialog = $state(false);
  let showRenameDialog = $state(false);
  let showDeleteDialog = $state(false);
  let showUnsavedChangesDialog = $state(false);

  let newNoteName = $state('');
  let newNoteNameForRename = $state('');

  let deleteKeyPressCount = $state(0);
  let deleteKeyResetTimeout: ReturnType<typeof setTimeout> | undefined = undefined;

  function openCreateDialog(query?: string, highlightedContent?: string): void {
    const currentQuery = query ?? context.query;
    const currentHighlightedContent = highlightedContent ?? context.highlightedContent;

    if (!currentHighlightedContent.trim() && currentQuery.trim()) {
      newNoteName = currentQuery.trim();
    } else {
      newNoteName = '';
    }
    showCreateDialog = true;
  }

  function closeCreateDialog(): void {
    showCreateDialog = false;
    newNoteName = '';
    deps.focusSearch();
  }

  function openRenameDialog(selectedNote?: string): void {
    const currentSelectedNote = selectedNote ?? context.selectedNote;
    if (currentSelectedNote) {
      newNoteNameForRename = currentSelectedNote.endsWith('.md')
        ? currentSelectedNote.slice(0, -3)
        : currentSelectedNote;
      showRenameDialog = true;
    }
  }

  function closeRenameDialog(): void {
    showRenameDialog = false;
    newNoteNameForRename = '';
    deps.focusSearch();
  }

  function openDeleteDialog(): void {
    showDeleteDialog = true;
    deleteKeyPressCount = 0;
  }

  function closeDeleteDialog(): void {
    showDeleteDialog = false;
    deleteKeyPressCount = 0;
    if (deleteKeyResetTimeout) {
      clearTimeout(deleteKeyResetTimeout);
      deleteKeyResetTimeout = undefined;
    }
    deps.focusSearch();
  }

  function handleDeleteKeyPress(onConfirmDelete: () => Promise<void>): void {
    deleteKeyPressCount++;

    if (deleteKeyResetTimeout) {
      clearTimeout(deleteKeyResetTimeout);
    }

    deleteKeyResetTimeout = setTimeout(() => {
      deleteKeyPressCount = 0;
      deleteKeyResetTimeout = undefined;
    }, 2000);

    if (deleteKeyPressCount >= 2) {
      deleteKeyPressCount = 0;
      if (deleteKeyResetTimeout) {
        clearTimeout(deleteKeyResetTimeout);
        deleteKeyResetTimeout = undefined;
      }
      onConfirmDelete();
    }
  }

  function openUnsavedChangesDialog(): void {
    showUnsavedChangesDialog = true;
  }

  function closeUnsavedChangesDialog(): void {
    showUnsavedChangesDialog = false;
    deps.focusSearch();
  }

  function showExitEditDialog(): void {
    openUnsavedChangesDialog();
  }

  async function handleSaveAndExit(saveAndExitNote: () => Promise<void>): Promise<void> {
    closeUnsavedChangesDialog();
    await saveAndExitNote();
  }

  function handleDiscardAndExit(exitEditMode: () => void): void {
    closeUnsavedChangesDialog();
    exitEditMode();
  }

  return {
    openCreateDialog,
    closeCreateDialog,
    openRenameDialog,
    closeRenameDialog,
    openDeleteDialog,
    closeDeleteDialog,
    openUnsavedChangesDialog,
    closeUnsavedChangesDialog,
    showExitEditDialog,
    handleSaveAndExit,
    handleDiscardAndExit,
    handleDeleteKeyPress,

    setNewNoteName(value: string): void {
      newNoteName = value;
    },

    setNewNoteNameForRename(value: string): void {
      newNoteNameForRename = value;
    },

    get showCreateDialog(): boolean {
      return showCreateDialog;
    },

    get showRenameDialog(): boolean {
      return showRenameDialog;
    },

    get showDeleteDialog(): boolean {
      return showDeleteDialog;
    },

    get showUnsavedChangesDialog(): boolean {
      return showUnsavedChangesDialog;
    },

    get newNoteName(): string {
      return newNoteName;
    },

    get newNoteNameForRename(): string {
      return newNoteNameForRename;
    },

    get deleteKeyPressCount(): number {
      return deleteKeyPressCount;
    }
  };
}

