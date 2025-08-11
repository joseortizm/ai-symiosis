/**
 * App Layer - Keyboard Actions
 * Keyboard shortcut handling with categorized action registry.
 * Maps key combinations to business logic functions across different UI contexts.
 */

import { invoke } from "@tauri-apps/api/core";

interface KeyboardActionDeps {
  setSelectedIndex: (value: number) => void;
  enterEditMode: () => Promise<void>;
  exitEditMode: () => void;
  saveAndExitNote: () => Promise<void>;
  showExitEditDialog: () => void;
  showDeleteDialog: () => void;
  showCreateDialog: () => void;
  showRenameDialog: () => void;
  openSettingsPane: () => Promise<void>;
  clearHighlights: () => void;
  clearSearch: () => void;
  focusSearch: () => void;
}

export interface AppState {
  isSearchInputFocused: boolean;
  isEditMode: boolean;
  isNoteContentFocused: boolean;
  selectedIndex: number;
  filteredNotes: string[];
  selectedNote: string | null;
  noteContentElement: HTMLElement | null;
  areHighlightsCleared: boolean;
  isEditorDirty: boolean;
  query: string;
}

export interface ActionContext {
  state: AppState;
  actions: KeyboardActionDeps;
}

export type ActionFunction = (context: ActionContext) => void | Promise<void>;

export type KeyMappings = Record<string, string>;

export interface ActionRegistry {
  [category: string]: {
    [actionName: string]: ActionFunction;
  };
}

export function createKeyboardActions(deps: KeyboardActionDeps) {
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
      focusSearch: ({ actions }: ActionContext) => {
        actions.focusSearch();
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
      exitEdit: ({ actions }: ActionContext) => {
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
        await actions.saveAndExitNote();
      },
    },

    notes: {
      openExternal: async ({ state }: ActionContext) => {
        if (state.selectedNote) {
          await invoke("open_note_in_editor", { noteName: state.selectedNote });
        }
      },
      openFolder: async ({ state }: ActionContext) => {
        if (state.selectedNote) {
          await invoke("open_note_folder", { noteName: state.selectedNote });
        }
      },
      refreshCache: async () => {
        await invoke("refresh_cache");
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
    },

    settings: {
      openSettings: async ({ actions }: ActionContext) => {
        await actions.openSettingsPane();
      },
    }
  };

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
      'Meta+,': 'settings.openSettings',
    },

    editMode: {
      'Escape': 'editing.smartExitEdit',
      'Ctrl+s': 'editing.save',
      'Meta+,': 'settings.openSettings',
    },

    noteContent: {
      'ArrowUp': 'scrolling.scrollUp',
      'ArrowDown': 'scrolling.scrollDown',
      'Ctrl+p': 'scrolling.scrollUp',
      'Ctrl+n': 'scrolling.scrollDown',
      'Escape': 'navigation.focusSearch',
      'e': 'editing.enterEdit',
      'Ctrl+x': 'notes.deleteNote',
      'Meta+,': 'settings.openSettings',
    },

    default: {
      'ArrowUp': 'navigation.moveUp',
      'ArrowDown': 'navigation.moveDown',
      'Enter': 'editing.enterEdit',
      'Ctrl+Enter': 'notes.createNote',
      'Ctrl+x': 'notes.deleteNote',
      'Escape': 'navigation.focusSearch',
      'Meta+,': 'settings.openSettings',
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

  function createKeyboardHandler(
    getState: () => AppState
  ): (event: KeyboardEvent) => Promise<void> {
    return async function handleKeydown(event: KeyboardEvent): Promise<void> {
      const state = getState();
      const context: ActionContext = { state, actions: deps };
      let handled = false;

      if (event.metaKey && event.key === ',') {
        event.preventDefault();
        await deps.openSettingsPane();
        return;
      }

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

  return {
    actionRegistry,
    keyMappings,
    createKeyboardHandler
  };
}
