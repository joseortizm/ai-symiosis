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

// Dialog visibility states
let showCreateDialog = $state(false);
let showRenameDialog = $state(false);
let showDeleteDialog = $state(false);

// Input values
let newNoteName = $state('');
let newNoteNameForRename = $state('');

// Delete dialog timer
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

export const dialogManager = {
  updateState(newState: Partial<DialogContext>): void {
    Object.assign(context, newState);
  },

  // Actions
  openCreateDialog,
  closeCreateDialog,
  openRenameDialog,
  closeRenameDialog,
  openDeleteDialog,
  closeDeleteDialog,
  handleDeleteKeyPress,

  // Setters
  setNewNoteName(value: string): void {
    newNoteName = value;
  },

  setNewNoteNameForRename(value: string): void {
    newNoteNameForRename = value;
  },

  // Reactive getters
  get showCreateDialog(): boolean {
    return showCreateDialog;
  },

  get showRenameDialog(): boolean {
    return showRenameDialog;
  },

  get showDeleteDialog(): boolean {
    return showDeleteDialog;
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
