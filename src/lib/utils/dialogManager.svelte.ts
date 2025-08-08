interface DialogContext {
  selectedNote: string | null;
  query: string;
  highlightedContent: string;
  searchElement: HTMLInputElement | null;
}

const context = $state<DialogContext>({
  selectedNote: null,
  query: '',
  highlightedContent: '',
  searchElement: null
});

let showCreateDialog = $state(false);
let showRenameDialog = $state(false);
let showDeleteDialog = $state(false);
let showUnsavedChangesDialog = $state(false);

let newNoteName = $state('');
let newNoteNameForRename = $state('');

let deleteKeyPressCount = $state(0);
let deleteKeyResetTimeout: number | undefined = undefined;

function openCreateDialog(): void {
  // Pre-fill with search query if no results and query exists
  if (!context.highlightedContent.trim() && context.query.trim()) {
    newNoteName = context.query.trim();
  } else {
    newNoteName = '';
  }
  showCreateDialog = true;
}

function closeCreateDialog(): void {
  showCreateDialog = false;
  newNoteName = '';
  context.searchElement?.focus();
}

function openRenameDialog(): void {
  if (context.selectedNote) {
    newNoteNameForRename = context.selectedNote.endsWith('.md')
      ? context.selectedNote.slice(0, -3)
      : context.selectedNote;
    showRenameDialog = true;
  }
}

function closeRenameDialog(): void {
  showRenameDialog = false;
  newNoteNameForRename = '';
  context.searchElement?.focus();
}

function openDeleteDialog(): void {
  showDeleteDialog = true;
  deleteKeyPressCount = 0;
  if (deleteKeyResetTimeout !== undefined) {
    clearTimeout(deleteKeyResetTimeout);
    deleteKeyResetTimeout = undefined;
  }
}

function closeDeleteDialog(): void {
  showDeleteDialog = false;
  deleteKeyPressCount = 0;
  if (deleteKeyResetTimeout !== undefined) {
    clearTimeout(deleteKeyResetTimeout);
    deleteKeyResetTimeout = undefined;
  }
  context.searchElement?.focus();
}

function handleDeleteKeyPress(onConfirmDelete: () => void): void {
  deleteKeyPressCount++;
  if (deleteKeyPressCount === 1) {
    deleteKeyResetTimeout = setTimeout(() => {
      deleteKeyPressCount = 0;
      deleteKeyResetTimeout = undefined;
    }, 2000);
  } else if (deleteKeyPressCount === 2) {
    if (deleteKeyResetTimeout !== undefined) {
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
  context.searchElement?.focus();
}

function showExitEditDialog(): void {
  openUnsavedChangesDialog();
}

function handleSaveAndExit(saveNote: () => void, exitEditMode: () => void): void {
  closeUnsavedChangesDialog();
  saveNote();
  exitEditMode();
}

function handleDiscardAndExit(exitEditMode: () => void): void {
  closeUnsavedChangesDialog();
  exitEditMode();
}

export const dialogManager = {
  updateState(newState: Partial<DialogContext>): void {
    Object.assign(context, newState);
  },

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
