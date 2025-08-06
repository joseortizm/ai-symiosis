import { getContext, setContext } from 'svelte';

// Unique context key
const key = {};

interface AppState {
  // Search state
  searchInput: string;
  query: string;
  isLoading: boolean;
  areHighlightsCleared: boolean;

  // Selection state
  filteredNotes: string[];
  selectedNote: string | null;
  selectedIndex: number;

  // Editor state
  noteContent: string;
  highlightedContent: string;
  isEditMode: boolean;
  editContent: string;
  isEditorDirty: boolean;
  nearestHeaderText: string;

  // Dialog state
  showDeleteDialog: boolean;
  showCreateDialog: boolean;
  showRenameDialog: boolean;
  showConfigDialog: boolean;
  showUnsavedChangesDialog: boolean;
  newNoteName: string;
  newNoteNameForRename: string;
  configContent: string;
  deleteKeyPressCount: number;
  deleteKeyResetTimeout: number | undefined;

  // UI state
  isSearchInputFocused: boolean;
  isNoteContentFocused: boolean;
  searchElement: HTMLInputElement | null;
  noteListElement: HTMLElement | null;
  noteContentElement: HTMLElement | null;
  renameDialogInputElement: HTMLInputElement | null;
  createDialogInputElement: HTMLInputElement | null;
}

export interface AppContext {
  // Reactive state object
  state: AppState;

  // Actions
  selectNote: (note: string, index: number) => void;
  deleteNote: () => Promise<void>;
  createNote: () => Promise<void>;
  renameNote: () => Promise<void>;
  saveNote: () => Promise<void>;
  enterEditMode: () => Promise<void>;
  exitEditMode: () => void;
  showExitEditDialog: () => void;
  handleSaveAndExit: () => void;
  handleDiscardAndExit: () => void;
  openCreateDialog: () => void;
  closeCreateDialog: () => void;
  openRenameDialog: () => void;
  closeRenameDialog: () => void;
  openDeleteDialog: () => void;
  closeDeleteDialog: () => void;
  openConfigDialog: () => Promise<void>;
  closeConfigDialog: () => void;
  saveConfig: () => Promise<void>;
  handleDeleteKeyPress: () => void;
  clearHighlights: () => void;
  clearSearch: () => void;
  invoke: any;
}

export function setAppContext(context: AppContext) {
  return setContext(key, context);
}

export function getAppContext(): AppContext {
  const context = getContext<AppContext>(key);
  if (!context) {
    throw new Error('App context not found. Make sure you are inside the app component.');
  }
  return context;
}
