/**
 * Version Service Tests (Factory-based - TDD)
 * Tests for version management operations including versions listing, content retrieval, and recovery.
 */

import { beforeEach, describe, expect, it, vi } from 'vitest'
import { resetAllMocks } from '../../test-utils'

// Mock tauri invoke at the top level
const mockInvoke = vi.fn()
vi.mock('@tauri-apps/api/core', () => ({
  invoke: mockInvoke,
}))

// Import after mocking
const { createVersionService } = await import(
  '../../../lib/services/versionService.svelte'
)

describe('versionService (factory-based - TDD)', () => {
  let versionService: ReturnType<typeof createVersionService>

  beforeEach(() => {
    resetAllMocks()
    versionService = createVersionService()
  })

  describe('getVersions', () => {
    it('should return empty array when no versions exist', async () => {
      mockInvoke.mockResolvedValue([])

      const result = await versionService.getVersions('test-note')

      expect(result.success).toBe(true)
      expect(result.versions).toEqual([])
      expect(mockInvoke).toHaveBeenCalledWith('get_note_versions', {
        noteName: 'test-note',
      })
    })

    it('should return versions when they exist', async () => {
      const mockVersions = [
        {
          filename: 'test-note.backup.1',
          backup_type: 'manual',
          timestamp: 1234567890,
          size: 1024,
          formatted_time: '2023-01-01 12:00:00',
        },
      ]
      mockInvoke.mockResolvedValue(mockVersions)

      const result = await versionService.getVersions('test-note')

      expect(result.success).toBe(true)
      expect(result.versions).toEqual(mockVersions)
    })

    it('should reject empty note name', async () => {
      const result = await versionService.getVersions('')

      expect(result.success).toBe(false)
      expect(result.error).toBe('Note name cannot be empty')
      expect(mockInvoke).not.toHaveBeenCalled()
    })

    it('should handle API errors', async () => {
      mockInvoke.mockRejectedValue('API Error')

      const result = await versionService.getVersions('test-note')

      expect(result.success).toBe(false)
      expect(result.error).toContain('Failed to load versions')
    })

    it('should update loading state during operation', async () => {
      mockInvoke.mockImplementation(
        () => new Promise((resolve) => setTimeout(resolve, 10))
      )

      const promise = versionService.getVersions('test-note')
      expect(versionService.isLoading).toBe(true)

      await promise
      expect(versionService.isLoading).toBe(false)
    })
  })

  describe('recoverVersion', () => {
    it('should recover version successfully', async () => {
      mockInvoke.mockResolvedValue(undefined)

      const result = await versionService.recoverVersion(
        'test-note',
        'test-note.backup.1'
      )

      expect(result.success).toBe(true)
      expect(mockInvoke).toHaveBeenCalledWith('recover_note_version', {
        noteName: 'test-note',
        versionFilename: 'test-note.backup.1',
      })
    })

    it('should reject empty note name', async () => {
      const result = await versionService.recoverVersion('', 'backup.1')

      expect(result.success).toBe(false)
      expect(result.error).toContain(
        'Note name and version filename are required'
      )
      expect(mockInvoke).not.toHaveBeenCalled()
    })

    it('should reject empty version filename', async () => {
      const result = await versionService.recoverVersion('test-note', '')

      expect(result.success).toBe(false)
      expect(result.error).toContain(
        'Note name and version filename are required'
      )
    })

    it('should handle recovery errors', async () => {
      mockInvoke.mockRejectedValue('Recovery failed')

      const result = await versionService.recoverVersion(
        'test-note',
        'backup.1'
      )

      expect(result.success).toBe(false)
      expect(result.error).toContain('Failed to recover version')
    })
  })

  describe('getVersionContent', () => {
    it('should return version content', async () => {
      mockInvoke.mockResolvedValue('# Test Content')

      const content = await versionService.getVersionContent('backup.1')

      expect(content).toBe('# Test Content')
      expect(mockInvoke).toHaveBeenCalledWith('get_version_content', {
        versionFilename: 'backup.1',
      })
    })

    it('should throw error when content loading fails', async () => {
      mockInvoke.mockRejectedValue('Content not found')

      await expect(
        versionService.getVersionContent('backup.1')
      ).rejects.toEqual('Content not found')
    })
  })

  describe('getDeletedFiles', () => {
    it('should return empty array when no deleted files exist', async () => {
      mockInvoke.mockResolvedValue([])

      const result = await versionService.getDeletedFiles()

      expect(result.success).toBe(true)
      expect(result.files).toEqual([])
    })

    it('should return deleted files when they exist', async () => {
      const mockFiles = [
        {
          filename: 'deleted-note',
          backup_filename: 'deleted-note.backup',
          deleted_at: '2023-01-01',
          timestamp: 1234567890,
        },
      ]
      mockInvoke.mockResolvedValue(mockFiles)

      const result = await versionService.getDeletedFiles()

      expect(result.success).toBe(true)
      expect(result.files).toEqual(mockFiles)
    })

    it('should handle errors loading deleted files', async () => {
      mockInvoke.mockRejectedValue('Database error')

      const result = await versionService.getDeletedFiles()

      expect(result.success).toBe(false)
      expect(result.error).toContain('Failed to load deleted files')
    })
  })

  describe('recoverDeletedFile', () => {
    it('should recover deleted file successfully', async () => {
      mockInvoke.mockResolvedValue(undefined)

      const result = await versionService.recoverDeletedFile(
        'deleted-note',
        'deleted-note.backup'
      )

      expect(result.success).toBe(true)
      expect(mockInvoke).toHaveBeenCalledWith('recover_deleted_file', {
        originalFilename: 'deleted-note',
        backupFilename: 'deleted-note.backup',
      })
    })

    it('should reject empty filenames', async () => {
      const result = await versionService.recoverDeletedFile('', 'backup')

      expect(result.success).toBe(false)
      expect(result.error).toContain(
        'Original filename and backup filename are required'
      )
      expect(mockInvoke).not.toHaveBeenCalled()
    })

    it('should handle recovery errors', async () => {
      mockInvoke.mockRejectedValue('Recovery failed')

      const result = await versionService.recoverDeletedFile(
        'deleted-note',
        'backup'
      )

      expect(result.success).toBe(false)
      expect(result.error).toContain('Failed to recover deleted file')
    })
  })

  describe('clearError', () => {
    it('should clear error state', async () => {
      mockInvoke.mockRejectedValue('Test error')
      await versionService.getVersions('test-note')
      expect(versionService.error).toBeTruthy()

      versionService.clearError()

      expect(versionService.error).toBeNull()
    })
  })

  describe('reactive state', () => {
    it('should expose loading state', () => {
      expect(versionService.isLoading).toBe(false)
    })

    it('should expose error state', () => {
      expect(versionService.error).toBeNull()
    })

    it('should expose lastOperation state', () => {
      expect(versionService.lastOperation).toBeNull()
    })

    it('should update lastOperation during different operations', async () => {
      mockInvoke.mockResolvedValue([])

      await versionService.getVersions('test-note')
      expect(versionService.lastOperation).toBe('list')

      await versionService.recoverVersion('test-note', 'backup')
      expect(versionService.lastOperation).toBe('recover')
    })
  })

  describe('edge cases', () => {
    it('clearError() after multiple failures should always reset error to null', async () => {
      mockInvoke.mockRejectedValue('First error')
      await versionService.getVersions('test-note')
      expect(versionService.error).toBeTruthy()

      mockInvoke.mockRejectedValue('Second error')
      await versionService.recoverVersion('test-note', 'backup')
      expect(versionService.error).toBeTruthy()

      versionService.clearError()
      expect(versionService.error).toBeNull()
    })

    it('operations with filenames containing spaces should succeed and pass arguments unchanged', async () => {
      mockInvoke.mockResolvedValue([])

      await versionService.getVersions('my note with spaces')

      expect(mockInvoke).toHaveBeenCalledWith('get_note_versions', {
        noteName: 'my note with spaces',
      })
    })

    it('operations with filenames containing unicode characters should succeed', async () => {
      mockInvoke.mockResolvedValue([])

      await versionService.getVersions('ノート 📝 émojis')

      expect(mockInvoke).toHaveBeenCalledWith('get_note_versions', {
        noteName: 'ノート 📝 émojis',
      })
    })

    it('overlapping calls should update lastOperation correctly to the most recent one', async () => {
      mockInvoke.mockImplementation(
        () => new Promise((resolve) => setTimeout(resolve, 10))
      )

      // Start first operation
      const promise1 = versionService.getVersions('test-note')
      expect(versionService.lastOperation).toBe('list')

      // Start second operation before first completes
      const promise2 = versionService.recoverVersion('test-note', 'backup')
      expect(versionService.lastOperation).toBe('recover')

      // Wait for both to complete
      await Promise.all([promise1, promise2])

      // Should reflect the most recent operation
      expect(versionService.lastOperation).toBe('recover')
    })

    it('recoverDeletedFile with special characters should pass arguments unchanged', async () => {
      mockInvoke.mockResolvedValue(undefined)

      await versionService.recoverDeletedFile(
        'file with spaces & émojis 📄.md',
        'backup-file with spaces & émojis 📄.backup'
      )

      expect(mockInvoke).toHaveBeenCalledWith('recover_deleted_file', {
        originalFilename: 'file with spaces & émojis 📄.md',
        backupFilename: 'backup-file with spaces & émojis 📄.backup',
      })
    })

    it('getVersionContent with special filename should pass unchanged', async () => {
      mockInvoke.mockResolvedValue('# Content')

      await versionService.getVersionContent(
        'backup with spaces & émojis 📄.backup'
      )

      expect(mockInvoke).toHaveBeenCalledWith('get_version_content', {
        versionFilename: 'backup with spaces & émojis 📄.backup',
      })
    })
  })
})
