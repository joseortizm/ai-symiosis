/**
 * App Layer - Application Coordinator
 * Central coordinator for app-wide state, actions, and effects.
 */

import { tick } from 'svelte'
import { listen } from '@tauri-apps/api/event'
import { createDialogManager } from '../core/dialogManager.svelte'
import { createContentManager } from '../core/contentManager.svelte'
import { createConfigManager as createConfigManager } from '../core/configManager.svelte'
import { createContentNavigationManager } from '../core/contentNavigationManager.svelte'
import { createProgressManager } from '../core/progressManager.svelte'
import { createSearchManager } from '../core/searchManager.svelte'
import { createEditorManager } from '../core/editorManager.svelte'
import { createFocusManager } from '../core/focusManager.svelte'
import { createVersionExplorerManager } from '../core/versionExplorerManager.svelte'
import { createRecentlyDeletedManager } from '../core/recentlyDeletedManager.svelte'
import { noteService } from '../services/noteService.svelte'
import { configService } from '../services/configService.svelte'
import { versionService } from '../services/versionService.svelte'
import { createNoteActions } from './actions/note.svelte'
import { createSearchActions } from './actions/search.svelte'
import { createSettingsActions } from './actions/settings.svelte'
import { createKeyboardActions } from './actions/keyboard.svelte'
import { setupAppEffects } from './effects/app.svelte'

// eslint-disable-next-line @typescript-eslint/no-empty-object-type
interface AppCoordinatorDeps {}

export interface AppState {
  readonly query: string
  readonly isLoading: boolean
  readonly filteredNotes: string[]
  readonly selectedNote: string | null
}

export interface AppActions {
  loadNoteContent: (note: string) => Promise<void>
  deleteNote: () => Promise<void>
  createNote: (noteName?: string) => Promise<void>
  renameNote: (newName?: string) => Promise<void>
  saveNote: () => Promise<void>
  saveAndExitNote: () => Promise<void>
  enterEditMode: () => Promise<void>
  exitEditMode: () => void
  refreshCacheAndUI: () => Promise<void>
  saveConfigAndRefresh: () => Promise<{ success: boolean; error?: string }>
}

export interface AppManagers {
  searchManager: ReturnType<
    typeof import('../core/searchManager.svelte').createSearchManager
  >
  editorManager: ReturnType<
    typeof import('../core/editorManager.svelte').createEditorManager
  >
  focusManager: ReturnType<
    typeof import('../core/focusManager.svelte').createFocusManager
  >
  contentManager: ReturnType<
    typeof import('../core/contentManager.svelte').createContentManager
  >
  dialogManager: ReturnType<
    typeof import('../core/dialogManager.svelte').createDialogManager
  >
  configManager: ReturnType<
    typeof import('../core/configManager.svelte').createConfigManager
  >
  contentNavigationManager: ReturnType<
    typeof import('../core/contentNavigationManager.svelte').createContentNavigationManager
  >
  progressManager: ReturnType<
    typeof import('../core/progressManager.svelte').createProgressManager
  >
  versionExplorerManager: ReturnType<
    typeof import('../core/versionExplorerManager.svelte').createVersionExplorerManager
  >
  recentlyDeletedManager: ReturnType<
    typeof import('../core/recentlyDeletedManager.svelte').createRecentlyDeletedManager
  >
}

export interface AppCoordinator {
  readonly query: string
  readonly isLoading: boolean
  readonly filteredNotes: string[]
  readonly selectedNote: string | null
  readonly keyboardActions: (event: KeyboardEvent) => Promise<void>
  readonly managers: AppManagers
  readonly state: AppState
  readonly actions: AppActions
  setupReactiveEffects(): () => void
  updateFilteredNotes(notes: string[]): void
  initialize(): Promise<() => void>
  handleSettingsClose(): void
}

export function createAppCoordinator(
  _deps: AppCoordinatorDeps
): AppCoordinator {
  const progressManager = createProgressManager()

  const searchManager = createSearchManager({
    noteService,
    progressManager,
  })

  const focusManager = createFocusManager()

  const contentNavigationManager = createContentNavigationManager({
    focusManager,
    searchManager,
  })

  const editorManager = createEditorManager({
    noteService,
    contentNavigationManager,
  })

  const dialogManager = createDialogManager({
    focusSearch: () => focusManager.focusSearch(),
  })

  const configManager = createConfigManager()

  const contentManager = createContentManager({
    noteService,
    searchManager,
    focusManager,
    contentNavigationManager,
  })

  const versionExplorerManager = createVersionExplorerManager({
    focusSearch: () => focusManager.focusSearch(),
    versionService,
    loadNoteContent,
  })

  let contentRequestController: AbortController | null = null
  let contentRequestSequence = 0

  let isFirstRun = false

  const isLoading = $derived(searchManager.isLoading)
  const filteredNotes = $derived(searchManager.filteredNotes)
  const query = $derived(searchManager.searchInput)

  const selectedNote = $derived.by(() => {
    const notes = filteredNotes
    let index = focusManager.selectedIndex

    if (notes.length === 0) {
      return null
    }

    if (index === -1 || index >= notes.length) {
      index = 0
    }

    return notes[index] || null
  })

  const noteActions = createNoteActions({
    noteService,
    searchManager,
    dialogManager,
    focusManager,
    editorManager,
    contentManager,
  })

  const searchActions = createSearchActions({
    searchManager,
    contentManager,
    focusManager,
    editorManager,
    contentNavigationManager,
  })

  const settingsActions = createSettingsActions({
    configService,
    focusManager,
  })

  function exitEditMode(): void {
    const exitHeaderText = editorManager.exitEditMode()
    if (exitHeaderText) {
      setTimeout(() => {
        contentNavigationManager.navigateToHeader(exitHeaderText)
      }, 100)
    }
    focusManager.focusSearch()
  }

  function handleSettingsClose(): void {
    configService.closePane()
    focusManager.focusSearch()

    if (isFirstRun) {
      // Delay to ensure settings dialog is fully closed
      setTimeout(() => {
        // Simulate Ctrl+? to show hints
        const event = new KeyboardEvent('keydown', {
          key: '?',
          ctrlKey: true,
          bubbles: true,
          cancelable: true,
        })
        document.dispatchEvent(event)
      }, 300)
    }
  }

  async function loadNoteContent(note: string): Promise<void> {
    if (contentRequestController) {
      contentRequestController.abort()
    }

    const currentSequence = ++contentRequestSequence

    contentNavigationManager.resetNavigation()

    if (!note) {
      if (currentSequence === contentRequestSequence) {
        contentManager.setNoteContent('')
      }
      return
    }

    const controller = new AbortController()
    contentRequestController = controller

    try {
      // Delegate to content manager - this handles both service call and state update
      await contentManager.refreshContent(note)

      // CRITICAL: Check both abort signal AND sequence to prevent race conditions
      if (
        !controller.signal.aborted &&
        currentSequence === contentRequestSequence
      ) {
        // Trigger navigation after content is loaded - only if this is still the latest request
        requestAnimationFrame(() => {
          // Double-check sequence in requestAnimationFrame since it's async
          if (currentSequence === contentRequestSequence) {
            contentManager.scrollToFirstMatch()
          }
        })
      }
    } catch (e) {
      // Only handle error if this is still the latest request and not aborted
      if (
        !controller.signal.aborted &&
        currentSequence === contentRequestSequence
      ) {
        console.error('Failed to load note content:', e)
        const errorMessage = String(e)
        contentManager.setNoteContent(`Error loading note: ${errorMessage}`)

        if (errorMessage.includes('Note not found')) {
          try {
            await refreshCacheAndUI()
          } catch (refreshError) {
            console.error('Auto-refresh failed:', refreshError)
          }
        }
      }
    }
  }

  async function saveAndExitNote(): Promise<void> {
    await noteActions.saveNote()
    exitEditMode()
    // An empty search shows notes in order
    // of most recent and we just saved it.
    focusManager.setSelectedIndex(0)
  }

  async function refreshCacheAndUI(): Promise<void> {
    await configService.refreshCache()
    await refreshUI()
  }

  const recentlyDeletedManager = createRecentlyDeletedManager({
    focusSearch: () => focusManager.focusSearch(),
    refreshCacheAndUI,
  })

  async function refreshUI(): Promise<void> {
    await searchManager.refreshSearch('')
    contentManager.setNoteContent('')
    focusManager.setSelectedIndex(0)
  }

  async function saveConfigAndRefresh(): Promise<{
    success: boolean
    error?: string
  }> {
    const result = await configService.save()

    if (result.success) {
      await searchManager.refreshSearch('')
      focusManager.focusSearch()
    }

    return result
  }

  const keyboardActions = createKeyboardActions({
    focusManager,
    contentNavigationManager,
    configManager,
    searchManager,
    contentManager,
    dialogManager,
    versionExplorerManager,
    recentlyDeletedManager,
    noteActions,
    settingsActions,
    noteService,
    appCoordinator: {
      loadNoteContent,
      exitEditMode,
      saveAndExitNote,
      refreshCacheAndUI,
    },
  })

  function setupReactiveEffects(): () => void {
    return setupAppEffects({
      getHideHighlights: () => contentNavigationManager.hideHighlights,
      focusManager,
      contentManager,
      searchManager,
      contentNavigationManager,
    })
  }

  return {
    setupReactiveEffects,
    handleSettingsClose,

    get query(): string {
      return query
    },
    get isLoading(): boolean {
      return isLoading
    },
    get filteredNotes(): string[] {
      return filteredNotes
    },
    get selectedNote(): string | null {
      return selectedNote
    },

    updateFilteredNotes: searchActions.updateFilteredNotes,

    get keyboardActions() {
      return keyboardActions.createKeyboardHandler(() => ({
        isSearchInputFocused: focusManager.isSearchInputFocused,
        isEditMode: editorManager.isEditMode,
        isNoteContentFocused: focusManager.isNoteContentFocused,
        filteredNotes: filteredNotes,
        selectedNote: selectedNote,
        noteContentElement: focusManager.noteContentElement,
        hideHighlights: contentNavigationManager.hideHighlights,
        isEditorDirty: editorManager.isDirty,
        query: query,
        isSettingsOpen: configService.isVisible,
        isAnyDialogOpen:
          dialogManager.showCreateDialog ||
          dialogManager.showRenameDialog ||
          dialogManager.showDeleteDialog ||
          dialogManager.showUnsavedChangesDialog ||
          versionExplorerManager.isVisible ||
          recentlyDeletedManager.isVisible,
      }))
    },

    get managers() {
      return {
        searchManager,
        editorManager,
        focusManager,
        contentManager,
        dialogManager,
        configManager,
        contentNavigationManager,
        progressManager,
        versionExplorerManager,
        recentlyDeletedManager,
      }
    },

    get state() {
      return {
        get query() {
          return query
        },
        get isLoading() {
          return isLoading
        },
        get filteredNotes() {
          return filteredNotes
        },
        get selectedNote() {
          return selectedNote
        },
      }
    },

    get actions() {
      return {
        loadNoteContent,
        deleteNote: () => noteActions.deleteNote(selectedNote),
        createNote: noteActions.createNote,
        renameNote: (newName?: string) =>
          noteActions.renameNote(selectedNote, newName),
        saveNote: () => noteActions.saveNote(),
        saveAndExitNote,
        enterEditMode: () => noteActions.enterEditMode(selectedNote!),
        exitEditMode,
        refreshCacheAndUI,
        saveConfigAndRefresh,
      }
    },

    async initialize(): Promise<() => void> {
      await tick()

      await configManager.initialize()

      // Set up search complete callback to load first note content
      searchManager.setSearchCompleteCallback(async (notes: string[]) => {
        if (notes.length > 0) {
          focusManager.setSelectedIndex(0)
          await loadNoteContent(notes[0])
        }
      })

      const unlisten = await listen('open-preferences', async () => {
        await settingsActions.openSettingsPane()
      })

      const unlistenCacheRefresh = await listen('cache-refreshed', async () => {
        await refreshUI()
      })

      const unlistenFirstRun = await listen('first-run-detected', () => {
        isFirstRun = true
      })

      // Database loading progress event listeners - MUST be set up before initialization
      const unlistenDbLoadingStart = await listen<string>(
        'db-loading-start',
        (event) => {
          progressManager.start(event.payload)
        }
      )

      const unlistenDbLoadingProgress = await listen<string>(
        'db-loading-progress',
        (event) => {
          progressManager.updateProgress(event.payload)
        }
      )

      const unlistenDbLoadingComplete = await listen(
        'db-loading-complete',
        () => {
          progressManager.complete()
        }
      )

      const unlistenDbLoadingError = await listen<string>(
        'db-loading-error',
        (event) => {
          progressManager.setError(event.payload)
        }
      )

      const configExists = await configService.exists()
      if (!configExists) {
        await settingsActions.openSettingsPane()
      } else {
        const result = await noteService.initializeDatabase()
        if (!result.success) {
          console.error('Failed to initialize notes:', result.error)
        }

        focusManager.focusSearch()
        const notes = await searchManager.searchImmediate('')
        if (notes.length > 0) {
          focusManager.setSelectedIndex(0)
          await loadNoteContent(notes[0])
        }
      }

      const cleanupEffects = setupReactiveEffects()

      return () => {
        searchManager.abort()
        if (contentRequestController) {
          contentRequestController.abort()
          contentRequestController = null
        }
        cleanupEffects()
        unlisten()
        unlistenCacheRefresh()
        unlistenFirstRun()
        unlistenDbLoadingStart()
        unlistenDbLoadingProgress()
        unlistenDbLoadingComplete()
        unlistenDbLoadingError()
        configManager.cleanup()
      }
    },
  }
}
