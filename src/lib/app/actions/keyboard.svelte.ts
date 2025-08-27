/**
 * App Layer - Keyboard Actions
 * Keyboard shortcut handling with categorized action registry.
 * Maps key combinations to business logic functions across different UI contexts.
 */

// Type definitions
export interface KeyboardActionDeps {
  focusManager: ReturnType<
    typeof import('../../core/focusManager.svelte').createFocusManager
  >
  contentNavigationManager: ReturnType<
    typeof import('../../core/contentNavigationManager.svelte').createContentNavigationManager
  >
  configStateManager: ReturnType<
    typeof import('../../core/configStateManager.svelte').createConfigStateManager
  >
  searchManager: ReturnType<
    typeof import('../../core/searchManager.svelte').createSearchManager
  >
  contentManager: ReturnType<
    typeof import('../../core/contentManager.svelte').createContentManager
  >
  dialogManager: ReturnType<
    typeof import('../../core/dialogManager.svelte').createDialogManager
  >
  noteActions: ReturnType<typeof import('./note.svelte').createNoteActions>
  settingsActions: ReturnType<
    typeof import('./settings.svelte').createSettingsActions
  >
  noteService: typeof import('../../services/noteService.svelte').noteService
  appCoordinator: {
    loadNoteContent: (note: string) => Promise<void>
    exitEditMode: () => void
    saveAndExitNote: () => Promise<void>
    refreshCacheAndUI: () => Promise<void>
  }
}

export interface AppState {
  isSearchInputFocused: boolean
  isEditMode: boolean
  isNoteContentFocused: boolean
  filteredNotes: string[]
  selectedNote: string | null
  noteContentElement: HTMLElement | null
  hideHighlights: boolean
  isEditorDirty: boolean
  query: string
  isSettingsOpen: boolean
  isAnyDialogOpen: boolean
}

export interface ActionContext {
  state: AppState
  actions: KeyboardActionDeps
}

export type ActionFunction = (context: ActionContext) => void | Promise<void>

export type KeyMappings = Record<string, string>

export interface ActionRegistry {
  [category: string]: {
    [actionName: string]: ActionFunction
  }
}

interface KeyboardActions {
  readonly actionRegistry: ActionRegistry
  readonly keyMappings: () => Record<string, KeyMappings>
  createKeyboardHandler(
    getState: () => AppState
  ): (event: KeyboardEvent) => Promise<void>
}

// Actions factory function
export function createKeyboardActions(
  deps: KeyboardActionDeps
): KeyboardActions {
  // Action registry organized by category
  const actionRegistry: ActionRegistry = {
    // Navigation actions
    navigation: {
      moveUp: ({ state, actions }: ActionContext) => {
        const newIndex = Math.max(0, actions.focusManager.selectedIndex - 1)
        const note = state.filteredNotes[newIndex]
        if (note) {
          actions.focusManager.setSelectedIndex(newIndex)
          void actions.appCoordinator.loadNoteContent(note)
        }
      },
      moveDown: ({ state, actions }: ActionContext) => {
        const maxIndex = state.filteredNotes.length - 1
        const newIndex = Math.min(
          maxIndex,
          actions.focusManager.selectedIndex + 1
        )
        const note = state.filteredNotes[newIndex]
        if (note) {
          actions.focusManager.setSelectedIndex(newIndex)
          void actions.appCoordinator.loadNoteContent(note)
        }
      },
      focusSearch: ({ actions }: ActionContext) => {
        actions.focusManager.focusSearch()
      },
      navigateNext: ({ actions }: ActionContext) => {
        actions.contentNavigationManager.navigateNext()
      },
      navigatePrevious: ({ actions }: ActionContext) => {
        actions.contentNavigationManager.navigatePrevious()
      },
      navigateCodeNext: ({ actions }: ActionContext) => {
        actions.contentNavigationManager.navigateCodeNext()
      },
      navigateCodePrevious: ({ actions }: ActionContext) => {
        actions.contentNavigationManager.navigateCodePrevious()
      },
      copyCurrentSection: async ({ actions }: ActionContext) => {
        await actions.contentNavigationManager.copyCurrentSection()
      },
    },

    scrolling: {
      scrollUpBy: ({ state }: ActionContext) => {
        state.noteContentElement?.scrollBy({
          top: -(state.noteContentElement.clientHeight * 0.4), // 40% up
          behavior: 'smooth',
        })
      },
      scrollDownBy: ({ state }: ActionContext) => {
        state.noteContentElement?.scrollBy({
          top: state.noteContentElement.clientHeight * 0.4, // 40% down
          behavior: 'smooth',
        })
      },
    },

    editing: {
      enterEdit: async ({ state, actions }: ActionContext) => {
        if (state.selectedNote && state.filteredNotes.length > 0) {
          await actions.noteActions.enterEditMode(state.selectedNote)
        }
      },
      exitEdit: ({ actions }: ActionContext) => {
        actions.appCoordinator.exitEditMode()
      },
      smartExitEdit: ({ state, actions }: ActionContext) => {
        if (state.isEditorDirty) {
          actions.dialogManager.showExitEditDialog()
        } else {
          actions.appCoordinator.exitEditMode()
        }
      },
      saveAndExit: async ({ actions }: ActionContext) => {
        await actions.appCoordinator.saveAndExitNote()
      },
    },

    notes: {
      openExternal: async ({ state, actions }: ActionContext) => {
        if (state.selectedNote) {
          await actions.noteService.openInEditor(state.selectedNote)
        }
      },
      openFolder: async ({ state, actions }: ActionContext) => {
        if (state.selectedNote) {
          await actions.noteService.openFolder(state.selectedNote)
        }
      },
      refreshCache: async ({ actions }: ActionContext) => {
        actions.appCoordinator.refreshCacheAndUI()
      },
      deleteNote: ({ state, actions }: ActionContext) => {
        if (state.selectedNote) {
          actions.dialogManager.openDeleteDialog()
        }
      },
      createNote: ({ actions }: ActionContext) => {
        actions.dialogManager.openCreateDialog()
      },
      renameNote: ({ state, actions }: ActionContext) => {
        if (state.selectedNote) {
          actions.dialogManager.openRenameDialog(state.selectedNote)
        }
      },
    },

    search: {
      handleEscape: ({ actions }: ActionContext) => {
        const escapeAction = actions.contentNavigationManager.handleEscape()

        switch (escapeAction) {
          case 'navigation_cleared':
            // Navigation was cleared, done
            break
          case 'highlights_cleared':
            // Highlights were cleared, done
            break
          case 'search_cleared':
            // Search was cleared, done
            break
          case 'focus_search':
            actions.focusManager.focusSearch()
            break
        }
      },
    },

    settings: {
      openSettings: async ({ actions }: ActionContext) => {
        await actions.settingsActions.openSettingsPane()
      },
    },
  }

  function getKeyMappings(): Record<string, KeyMappings> {
    const shortcuts = deps.configStateManager.shortcuts

    return {
      searchInput: {
        Enter: 'editing.enterEdit',
        [shortcuts.create_note]: 'notes.createNote',
        [shortcuts.rename_note]: 'notes.renameNote',
        [shortcuts.open_external]: 'notes.openExternal',
        [shortcuts.open_folder]: 'notes.openFolder',
        [shortcuts.refresh_cache]: 'notes.refreshCache',
        [shortcuts.delete_note]: 'notes.deleteNote',
        [shortcuts.scroll_up]: 'scrolling.scrollUpBy',
        [shortcuts.scroll_down]: 'scrolling.scrollDownBy',
        ArrowUp: 'navigation.moveUp',
        ArrowDown: 'navigation.moveDown',
        [shortcuts.vim_up]: 'navigation.moveUp',
        [shortcuts.vim_down]: 'navigation.moveDown',
        [shortcuts.navigate_previous]: 'navigation.navigatePrevious',
        [shortcuts.navigate_next]: 'navigation.navigateNext',
        [shortcuts.navigate_code_previous]: 'navigation.navigateCodePrevious',
        [shortcuts.navigate_code_next]: 'navigation.navigateCodeNext',
        [shortcuts.copy_current_section]: 'navigation.copyCurrentSection',
        Escape: 'search.handleEscape',
        [shortcuts.open_settings]: 'settings.openSettings',
      },

      editMode: {
        Escape: 'editing.smartExitEdit',
        [shortcuts.save_and_exit]: 'editing.saveAndExit',
        [shortcuts.open_settings]: 'settings.openSettings',
      },

      noteContent: {
        Escape: 'navigation.focusSearch',
        [shortcuts.navigate_previous]: 'navigation.navigatePrevious',
        [shortcuts.navigate_next]: 'navigation.navigateNext',
        [shortcuts.navigate_code_previous]: 'navigation.navigateCodePrevious',
        [shortcuts.navigate_code_next]: 'navigation.navigateCodeNext',
        [shortcuts.copy_current_section]: 'navigation.copyCurrentSection',
      },

      default: {
        ArrowUp: 'navigation.moveUp',
        ArrowDown: 'navigation.moveDown',
        Enter: 'editing.enterEdit',
        [shortcuts.create_note]: 'notes.createNote',
        [shortcuts.delete_note]: 'notes.deleteNote',
        Escape: 'navigation.focusSearch',
        [shortcuts.open_settings]: 'settings.openSettings',
      },
    }
  }

  function formatKeyCombo(event: KeyboardEvent): string {
    const modifiers: string[] = []
    if (event.ctrlKey) modifiers.push('Ctrl')
    if (event.altKey) modifiers.push('Alt')
    if (event.shiftKey) modifiers.push('Shift')
    if (event.metaKey) modifiers.push('Meta')

    return modifiers.length > 0
      ? `${modifiers.join('+')}+${event.key}`
      : event.key
  }

  async function handleKeyAction(
    mappings: KeyMappings,
    event: KeyboardEvent,
    context: ActionContext
  ): Promise<boolean> {
    const keyString = formatKeyCombo(event)
    const actionPath = mappings[keyString]

    if (actionPath) {
      event.preventDefault()

      const [category, actionName] = actionPath.split('.')
      const action = actionRegistry[category]?.[actionName]

      if (action) {
        await action(context)
        return true
      } else {
        console.warn(`Action not found: ${actionPath}`)
      }
    }
    return false
  }

  function createKeyboardHandler(
    getState: () => AppState
  ): (event: KeyboardEvent) => Promise<void> {
    return async function handleKeydown(event: KeyboardEvent): Promise<void> {
      const state = getState()
      const context: ActionContext = { state, actions: deps }
      if (state.isSettingsOpen || state.isAnyDialogOpen) {
        if (event.key === 'Escape') {
          return
        }
        return
      }

      if (event.metaKey && event.key === ',') {
        event.preventDefault()
        await deps.settingsActions.openSettingsPane()
        return
      }

      const keyMappings = getKeyMappings()

      if (state.isSearchInputFocused) {
        await handleKeyAction(keyMappings.searchInput, event, context)
      } else if (state.isEditMode) {
        await handleKeyAction(keyMappings.editMode, event, context)
      } else if (state.isNoteContentFocused && !state.isEditMode) {
        await handleKeyAction(keyMappings.noteContent, event, context)
      } else if (state.filteredNotes.length > 0) {
        await handleKeyAction(keyMappings.default, event, context)
      }
    }
  }

  return {
    actionRegistry,
    keyMappings: getKeyMappings,
    createKeyboardHandler,
  }
}
