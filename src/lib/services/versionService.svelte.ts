/**
 * Service Layer - Version Service
 * Reactive operations for note version management with loading state.
 * Handles API calls to Rust backend for version listing, content retrieval, and recovery.
 */

import { invoke } from '@tauri-apps/api/core'

// Type definitions
export interface NoteVersion {
  filename: string
  backup_type: string
  timestamp: number
  size: number
  formatted_time: string
}

// Service factory function
export function createVersionService() {
  const state = $state({
    isLoading: false,
    error: null as string | null,
    lastOperation: null as 'list' | 'recover' | null,
  })

  // Private helper functions
  function clearError(): void {
    state.error = null
  }

  // Version operations with loading state
  async function getVersions(
    noteName: string
  ): Promise<{ success: boolean; versions?: NoteVersion[]; error?: string }> {
    if (!noteName.trim()) {
      return { success: false, error: 'Note name cannot be empty' }
    }

    state.isLoading = true
    state.error = null
    state.lastOperation = 'list'

    try {
      const versions = await invoke<NoteVersion[]>('get_note_versions', {
        noteName,
      })
      return { success: true, versions: versions || [] }
    } catch (e) {
      const error = `Failed to load versions: ${e}`
      state.error = error
      console.error('Failed to load versions:', e)
      return { success: false, error }
    } finally {
      state.isLoading = false
    }
  }

  async function recoverVersion(
    noteName: string,
    versionFilename: string
  ): Promise<{ success: boolean; error?: string }> {
    if (!noteName.trim() || !versionFilename.trim()) {
      return {
        success: false,
        error: 'Note name and version filename are required',
      }
    }

    state.isLoading = true
    state.error = null
    state.lastOperation = 'recover'

    try {
      await invoke<void>('recover_note_version', {
        noteName,
        versionFilename,
      })
      return { success: true }
    } catch (e) {
      const error = `Failed to recover version: ${e}`
      state.error = error
      console.error('Failed to recover version:', e)
      return { success: false, error }
    } finally {
      state.isLoading = false
    }
  }

  // Simple operations that throw (no loading state needed)
  async function getVersionContent(versionFilename: string): Promise<string> {
    try {
      return await invoke<string>('get_version_content', { versionFilename })
    } catch (e) {
      console.error('Failed to get version content:', e)
      throw e
    }
  }

  // Public API
  return {
    // Version operations
    getVersions,
    getVersionContent,
    recoverVersion,

    // Utility functions
    clearError,

    // State getters
    get isLoading() {
      return state.isLoading
    },
    get error() {
      return state.error
    },
    get lastOperation() {
      return state.lastOperation
    },
  }
}

// Global service instance
export const versionService = createVersionService()
