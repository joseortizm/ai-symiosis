import { focusManager } from './focusManager.svelte';

interface DialogContext {
  selectedNote: string | null;
  query: string;
  highlightedContent: string;
}

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
  // Use provided parameters or fallback to context
  const currentQuery = query ?? context.query;
  const currentHighlightedContent = highlightedContent ?? context.highlightedContent;
  
  // Pre-fill with search query if no results and query exists
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
  focusManager.focusSearch();
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
  focusManager.focusSearch();
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
  focusManager.focusSearch();
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
  focusManager.focusSearch();
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
