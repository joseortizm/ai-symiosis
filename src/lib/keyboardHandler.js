const actionRegistry = {
  navigation: {
    moveUp: ({ state, actions }) => {
      const newIndex = Math.max(0, state.selectedIndex - 1);
      actions.setSelectedIndex(newIndex);
    },
    moveDown: ({ state, actions }) => {
      const maxIndex = state.filteredNotes.length - 1;
      const newIndex = Math.min(maxIndex, state.selectedIndex + 1);
      actions.setSelectedIndex(newIndex);
    },
    focusSearch: ({ state }) => {
      state.searchElement?.focus();
    },
  },

  scrolling: {
    scrollUp: ({ state }) => {
      state.noteContentElement?.scrollBy({
        top: -50,
        behavior: 'smooth'
      });
    },
    scrollDown: ({ state }) => {
      state.noteContentElement?.scrollBy({
        top: 50,
        behavior: 'smooth'
      });
    },
    scrollUp200: ({ state }) => {
      state.noteContentElement?.scrollBy({
        top: -200,
        behavior: 'smooth'
      });
    },
    scrollDown200: ({ state }) => {
      state.noteContentElement?.scrollBy({
        top: 200,
        behavior: 'smooth'
      });
    },
  },

  editing: {
    enterEdit: async ({ state, actions }) => {
      if (state.selectedNote && state.filteredNotes.length > 0) {
        await actions.enterEditMode();
      }
    },
    exitEdit: ({ state, actions }) => {
      actions.exitEditMode();
      state.searchElement?.focus();
    },
    save: async ({ actions }) => {
      await actions.saveNote();
    },
  },

  notes: {
    openExternal: async ({ state, actions }) => {
      if (state.selectedNote) {
        await actions.invoke("open_note_in_editor", { noteName: state.selectedNote });
      }
    },
    refreshCache: async ({ state, actions }) => {
      await actions.invoke("refresh_cache");
    },
    deleteNote: ({ state, actions }) => {
      if (state.selectedNote) {
        actions.showDeleteDialog();
      }
    },
    createNote: ({ actions }) => {
      actions.showCreateDialog();
    },
    renameNote: ({ actions }) => {
      actions.showRenameDialog();
    },
  },

  search: {
    clearHighlights: ({ state, actions }) => {
      if (state.query.trim() && !state.highlightsCleared) {
        actions.clearHighlights();
      } else if (state.highlightsCleared || !state.query.trim()) {
        actions.clearSearch();
      }
    },
  }
};

// Key mappings for each mode - these reference the actions object above
const keyMappings = {
  searchInput: {
    'Enter': 'editing.enterEdit',
    'Ctrl+Enter': 'notes.createNote',
    'Ctrl+n': 'notes.createNote',
    'Ctrl+m': 'notes.renameNote',
    'Ctrl+o': 'notes.openExternal',
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
    'Escape': 'editing.exitEdit',
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
    'Ctrl+Enter': 'notes.createNote',
    'Ctrl+n': 'notes.createNote',
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

function getKeyString(event) {
  const modifiers = [];
  if (event.ctrlKey) modifiers.push('Ctrl');
  if (event.altKey) modifiers.push('Alt');
  if (event.shiftKey) modifiers.push('Shift');
  if (event.metaKey) modifiers.push('Meta');

  return modifiers.length > 0
    ? `${modifiers.join('+')}+${event.key}`
    : event.key;
}

async function executeKeyAction(mappings, event, context) {
  const keyString = getKeyString(event);
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

export function createKeyboardHandler(getState, actions) {
  return async function handleKeydown(event) {
    // Get fresh state each time the handler runs
    const state = getState();
    const context = { state, actions };

    let handled = false;

    if (state.isSearchInputFocused) {
      handled = await executeKeyAction(keyMappings.searchInput, event, context);
    } else if (state.isEditMode) {
      handled = await executeKeyAction(keyMappings.editMode, event, context);
    } else if (state.isNoteContentFocused && !state.isEditMode) {
      handled = await executeKeyAction(keyMappings.noteContent, event, context);
    } else if (state.filteredNotes.length > 0) {
      handled = await executeKeyAction(keyMappings.default, event, context);
    }
  };
}
