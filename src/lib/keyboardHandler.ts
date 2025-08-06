export interface AppState {
  isSearchInputFocused: boolean;
  isEditMode: boolean;
  isNoteContentFocused: boolean;
  selectedIndex: number;
  filteredNotes: string[];
  selectedNote: string | null;
  noteContentElement: HTMLElement | null;
  searchElement: HTMLInputElement | null;
  query: string;
  areHighlightsCleared: boolean;
  showConfigDialog: boolean;
  isEditorDirty: boolean;
}

export interface Actions {
  setSelectedIndex: (value: number) => void;
  enterEditMode: () => Promise<void>;
  exitEditMode: () => void;
  showExitEditDialog: () => void;
  saveNote: () => Promise<void>;
  invoke: (command: string, args?: Record<string, unknown>) => Promise<unknown>;
  showDeleteDialog: () => void;
  showCreateDialog: () => void;
  showRenameDialog: () => void;
  clearHighlights: () => void;
  clearSearch: () => void;
}

export interface ActionContext {
  state: AppState;
  actions: Actions;
}

export type ActionFunction = (context: ActionContext) => void | Promise<void>;

export type KeyMappings = Record<string, string>;

export interface ActionRegistry {
  [category: string]: {
    [actionName: string]: ActionFunction;
  };
}

const actionRegistry: ActionRegistry = {
  navigation: {
    moveUp: ({ state, actions }: ActionContext) => {
      const newIndex = Math.max(0, state.selectedIndex - 1);
      actions.setSelectedIndex(newIndex);
    },
    moveDown: ({ state, actions }: ActionContext) => {
      const maxIndex = state.filteredNotes.length - 1;
      const newIndex = Math.min(maxIndex, state.selectedIndex + 1);
      actions.setSelectedIndex(newIndex);
    },
    focusSearch: ({ state }: ActionContext) => {
      state.searchElement?.focus();
    },
  },

  scrolling: {
    scrollUp: ({ state }: ActionContext) => {
      state.noteContentElement?.scrollBy({
        top: -50,
        behavior: 'smooth'
      });
    },
    scrollDown: ({ state }: ActionContext) => {
      state.noteContentElement?.scrollBy({
        top: 50,
        behavior: 'smooth'
      });
    },
    scrollUp200: ({ state }: ActionContext) => {
      state.noteContentElement?.scrollBy({
        top: -200,
        behavior: 'smooth'
      });
    },
    scrollDown200: ({ state }: ActionContext) => {
      state.noteContentElement?.scrollBy({
        top: 200,
        behavior: 'smooth'
      });
    },
  },

  editing: {
    enterEdit: async ({ state, actions }: ActionContext) => {
      if (state.selectedNote && state.filteredNotes.length > 0) {
        await actions.enterEditMode();
      }
    },
    exitEdit: ({ state, actions }: ActionContext) => {
      actions.exitEditMode();
    },
    smartExitEdit: ({ state, actions }: ActionContext) => {
      if (state.isEditorDirty) {
        actions.showExitEditDialog();
      } else {
        actions.exitEditMode();
      }
    },
    save: async ({ actions }: ActionContext) => {
      await actions.saveNote();
    },
  },

  notes: {
    openExternal: async ({ state, actions }: ActionContext) => {
      if (state.selectedNote) {
        await actions.invoke("open_note_in_editor", { noteName: state.selectedNote });
      }
    },
    openFolder: async ({ state, actions }: ActionContext) => {
      if (state.selectedNote) {
        await actions.invoke("open_note_folder", { noteName: state.selectedNote });
      }
    },
    refreshCache: async ({ state, actions }: ActionContext) => {
      await actions.invoke("refresh_cache");
    },
    deleteNote: ({ state, actions }: ActionContext) => {
      if (state.selectedNote) {
        actions.showDeleteDialog();
      }
    },
    createNote: ({ actions }: ActionContext) => {
      actions.showCreateDialog();
    },
    renameNote: ({ actions }: ActionContext) => {
      actions.showRenameDialog();
    },
  },

  search: {
    clearHighlights: ({ state, actions }: ActionContext) => {
      if (state.query.trim() && !state.areHighlightsCleared) {
        actions.clearHighlights();
      } else if (state.areHighlightsCleared || !state.query.trim()) {
        actions.clearSearch();
      }
    },
  }
};

// Key mappings for each mode - these reference the actions object above
const keyMappings: Record<string, KeyMappings> = {
  searchInput: {
    'Enter': 'editing.enterEdit',
    'Ctrl+Enter': 'notes.createNote',
    'Ctrl+n': 'notes.createNote',
    'Ctrl+m': 'notes.renameNote',
    'Ctrl+o': 'notes.openExternal',
    'Ctrl+f': 'notes.openFolder',
    'Ctrl+r': 'notes.refreshCache',
    'Ctrl+x': 'notes.deleteNote',
    'Ctrl+u': 'scrolling.scrollUp200',
    'Ctrl+d': 'scrolling.scrollDown200',
    'ArrowUp': 'navigation.moveUp',
    'ArrowDown': 'navigation.moveDown',
    'Ctrl+k': 'navigation.moveUp',
    'Ctrl+j': 'navigation.moveDown',
    'Escape': 'search.clearHighlights',
  },

  editMode: {
    'Escape': 'editing.smartExitEdit',
    'Ctrl+s': 'editing.save',
  },

  noteContent: {
    'ArrowUp': 'scrolling.scrollUp',
    'ArrowDown': 'scrolling.scrollDown',
    'Ctrl+p': 'scrolling.scrollUp',
    'Ctrl+n': 'scrolling.scrollDown',
    'Escape': 'navigation.focusSearch',
    'e': 'editing.enterEdit',
    'Ctrl+x': 'notes.deleteNote',
  },

  default: {
    'ArrowUp': 'navigation.moveUp',
    'ArrowDown': 'navigation.moveDown',
    'Enter': 'editing.enterEdit',
    'Ctrl+Enter': 'notes.createNote',
    'Ctrl+x': 'notes.deleteNote',
    'Escape': 'navigation.focusSearch',
  }
};

function formatKeyCombo(event: KeyboardEvent): string {
  const modifiers: string[] = [];
  if (event.ctrlKey) modifiers.push('Ctrl');
  if (event.altKey) modifiers.push('Alt');
  if (event.shiftKey) modifiers.push('Shift');
  if (event.metaKey) modifiers.push('Meta');

  return modifiers.length > 0
    ? `${modifiers.join('+')}+${event.key}`
    : event.key;
}

async function handleKeyAction(
  mappings: KeyMappings,
  event: KeyboardEvent,
  context: ActionContext
): Promise<boolean> {
  const keyString = formatKeyCombo(event);
  const actionPath = mappings[keyString];

  if (actionPath) {
    event.preventDefault();

    // Resolve the action from the registry using the path
    const [category, actionName] = actionPath.split('.');
    const action = actionRegistry[category]?.[actionName];

    if (action) {
      await action(context);
      return true;
    } else {
      console.warn(`Action not found: ${actionPath}`);
    }
  }
  return false;
}

export function createKeyboardHandler(
  getState: () => AppState,
  actions: Actions
): (event: KeyboardEvent) => Promise<void> {
  return async function handleKeydown(event: KeyboardEvent): Promise<void> {
    const state = getState();

    if (state.showConfigDialog) return;

    const context: ActionContext = { state, actions };
    let handled = false;

    if (state.isSearchInputFocused) {
      handled = await handleKeyAction(keyMappings.searchInput, event, context);
    } else if (state.isEditMode) {
      handled = await handleKeyAction(keyMappings.editMode, event, context);
    } else if (state.isNoteContentFocused && !state.isEditMode) {
      handled = await handleKeyAction(keyMappings.noteContent, event, context);
    } else if (state.filteredNotes.length > 0) {
      handled = await handleKeyAction(keyMappings.default, event, context);
    }
  };
}
