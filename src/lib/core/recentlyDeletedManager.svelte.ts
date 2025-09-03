/**
 * Core Layer - Recently Deleted Manager
 * Modal state for recently deleted files functionality.
 * Handles dialog visibility, selection state, and coordinates with backend services.
 */

import type { DeletedFile } from '../services/versionService.svelte'
import { versionService } from '../services/versionService.svelte'

interface RecentlyDeletedState {
  isVisible: boolean
  files: DeletedFile[]
  selectedIndex: number
  isLoading: boolean
  error: string | null
}

export interface RecentlyDeletedManagerDeps {
  focusSearch: () => void
  refreshCacheAndUI: () => Promise<void>
}

export interface RecentlyDeletedManager {
  readonly isVisible: boolean
  readonly files: DeletedFile[]
  readonly selectedIndex: number
  readonly isLoading: boolean
  readonly error: string | null
  openDialog(): Promise<void>
  closeDialog(): void
  selectFile(index: number): void
  navigateUp(): void
  navigateDown(): void
  recoverFile(filename: string): Promise<void>
}

// Manager factory function
export function createRecentlyDeletedManager(
  deps: RecentlyDeletedManagerDeps
): RecentlyDeletedManager {
  const state = $state<RecentlyDeletedState>({
    isVisible: false,
    files: [],
    selectedIndex: 0,
    isLoading: false,
    error: null,
  })

  // Dialog operations
  async function openDialog(): Promise<void> {
    state.isVisible = true
    state.selectedIndex = 0
    state.error = null

    await loadDeletedFiles()
  }

  function closeDialog(): void {
    state.isVisible = false
    state.files = []
    state.selectedIndex = 0
    state.error = null
    deps.focusSearch()
  }

  // File loading - private helper
  async function loadDeletedFiles(): Promise<void> {
    state.isLoading = true
    state.error = null

    try {
      const result = await versionService.getDeletedFiles()

      if (result.success && result.files) {
        state.files = result.files

        // Auto-select first file if available
        if (state.files.length > 0) {
          state.selectedIndex = 0
        }
      } else {
        state.error = result.error || 'Failed to load deleted files'
        state.files = []
      }
    } catch (err) {
      state.error = `Failed to load deleted files: ${err}`
      state.files = []
    } finally {
      state.isLoading = false
    }
  }

  // Navigation operations
  function selectFile(index: number): void {
    if (index >= 0 && index < state.files.length) {
      state.selectedIndex = index
    }
  }

  function navigateUp(): void {
    if (state.selectedIndex > 0) {
      state.selectedIndex = state.selectedIndex - 1
    }
  }

  function navigateDown(): void {
    if (state.selectedIndex < state.files.length - 1) {
      state.selectedIndex = state.selectedIndex + 1
    }
  }

  // Recovery operations
  async function recoverFile(filename: string): Promise<void> {
    state.isLoading = true
    state.error = null

    try {
      // Find the file in our list to get the backup filename
      const deletedFile = state.files.find((f) => f.filename === filename)
      if (!deletedFile) {
        throw new Error(`File not found in deleted files list: ${filename}`)
      }

      const result = await versionService.recoverDeletedFile(
        deletedFile.filename,
        deletedFile.backup_filename
      )

      if (result.success) {
        // Remove recovered file from list
        state.files = state.files.filter((f) => f.filename !== filename)

        // Adjust selected index if needed
        if (state.selectedIndex >= state.files.length) {
          state.selectedIndex = Math.max(0, state.files.length - 1)
        }

        // Refresh the database and UI to show the recovered file
        await deps.refreshCacheAndUI()

        // Close dialog if no more files
        if (state.files.length === 0) {
          closeDialog()
        }
      } else {
        state.error = result.error || 'Failed to recover file'
      }
    } catch (err) {
      state.error = `Failed to recover file: ${err}`
    } finally {
      state.isLoading = false
    }
  }

  return {
    get isVisible() {
      return state.isVisible
    },
    get files() {
      return state.files
    },
    get selectedIndex() {
      return state.selectedIndex
    },
    get isLoading() {
      return state.isLoading
    },
    get error() {
      return state.error
    },
    openDialog,
    closeDialog,
    selectFile,
    navigateUp,
    navigateDown,
    recoverFile,
  }
}
