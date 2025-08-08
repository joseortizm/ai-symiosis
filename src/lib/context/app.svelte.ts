import { getContext, setContext } from 'svelte';

const key = {};

interface AppState {
  searchInput: string;
  query: string;
  isLoading: boolean;
  areHighlightsCleared: boolean;

  filteredNotes: string[];
  selectedNote: string | null;
  selectedIndex: number;

  noteContent: string;
  highlightedContent: string;
  isEditMode: boolean;
  editContent: string;
  isEditorDirty: boolean;
  nearestHeaderText: string;

  isSearchInputFocused: boolean;
  isNoteContentFocused: boolean;
  searchElement: HTMLInputElement | null;
  noteListElement: HTMLElement | null;
  noteContentElement: HTMLElement | null;
}

export interface AppContext {
  state: AppState;

  selectNote: (note: string, index: number) => void;
  deleteNote: () => Promise<void>;
  createNote: (noteNameParam?: string) => Promise<void>;
  renameNote: (newNameParam?: string) => Promise<void>;
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
  openSettingsPane: () => Promise<void>;
  closeSettingsPane: () => void;
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
