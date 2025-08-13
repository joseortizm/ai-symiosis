import { describe, it, expect, beforeEach, vi } from 'vitest'
import { mockSearchManager, resetAllMocks } from '../../test-utils'
import type {
  DialogManager,
  DialogManagerDeps,
} from '../../../lib/core/dialogManager.svelte'
import { createDialogManager } from '../../../lib/core/dialogManager.svelte'

describe('dialogManager (factory-based - TDD)', () => {
  let dialogManager: DialogManager
  let mockFocusCallback: () => void

  beforeEach(() => {
    resetAllMocks()
    mockFocusCallback = vi.fn()
    dialogManager = createDialogManager({ focusSearch: mockFocusCallback })
  })

  it('should create dialogManager with focus callback', () => {
    expect(dialogManager).toBeDefined()
    expect(typeof dialogManager.openCreateDialog).toBe('function')
    expect(typeof dialogManager.closeCreateDialog).toBe('function')
  })

  it('should call focus callback when closing create dialog', () => {
    dialogManager.openCreateDialog()
    dialogManager.closeCreateDialog()

    expect(mockFocusCallback).toHaveBeenCalled()
  })

  it('should call focus callback when closing rename dialog', () => {
    dialogManager.openRenameDialog()
    dialogManager.closeRenameDialog()

    expect(mockFocusCallback).toHaveBeenCalled()
  })

  it('should call focus callback when closing delete dialog', () => {
    dialogManager.openDeleteDialog()
    dialogManager.closeDeleteDialog()

    expect(mockFocusCallback).toHaveBeenCalled()
  })

  it('should call focus callback when closing unsaved changes dialog', () => {
    dialogManager.openUnsavedChangesDialog()
    dialogManager.closeUnsavedChangesDialog()

    expect(mockFocusCallback).toHaveBeenCalled()
  })

  describe('dialog state management', () => {
    it('should track create dialog state', () => {
      expect(dialogManager.showCreateDialog).toBe(false)

      dialogManager.openCreateDialog()
      expect(dialogManager.showCreateDialog).toBe(true)

      dialogManager.closeCreateDialog()
      expect(dialogManager.showCreateDialog).toBe(false)
    })

    it('should track rename dialog state', () => {
      expect(dialogManager.showRenameDialog).toBe(false)

      // openRenameDialog requires a selected note to open
      dialogManager.openRenameDialog('test-note.md')
      expect(dialogManager.showRenameDialog).toBe(true)

      dialogManager.closeRenameDialog()
      expect(dialogManager.showRenameDialog).toBe(false)
    })

    it('should track delete dialog state', () => {
      expect(dialogManager.showDeleteDialog).toBe(false)

      dialogManager.openDeleteDialog()
      expect(dialogManager.showDeleteDialog).toBe(true)

      dialogManager.closeDeleteDialog()
      expect(dialogManager.showDeleteDialog).toBe(false)
    })

    it('should track unsaved changes dialog state', () => {
      expect(dialogManager.showUnsavedChangesDialog).toBe(false)

      dialogManager.openUnsavedChangesDialog()
      expect(dialogManager.showUnsavedChangesDialog).toBe(true)

      dialogManager.closeUnsavedChangesDialog()
      expect(dialogManager.showUnsavedChangesDialog).toBe(false)
    })
  })

  describe('delete key press timing and cleanup', () => {
    beforeEach(() => {
      vi.useFakeTimers()
    })

    afterEach(() => {
      vi.runOnlyPendingTimers()
      vi.useRealTimers()
    })

    it('should reset delete key count after timeout', () => {
      const mockOnConfirmDelete = vi.fn()

      dialogManager.handleDeleteKeyPress(mockOnConfirmDelete)
      expect(dialogManager.deleteKeyPressCount).toBe(1)

      vi.advanceTimersByTime(2000)
      expect(dialogManager.deleteKeyPressCount).toBe(0)
      expect(mockOnConfirmDelete).not.toHaveBeenCalled()
    })

    it('should trigger delete on double key press within timeout', () => {
      const mockOnConfirmDelete = vi.fn()

      dialogManager.handleDeleteKeyPress(mockOnConfirmDelete)
      dialogManager.handleDeleteKeyPress(mockOnConfirmDelete)

      expect(dialogManager.deleteKeyPressCount).toBe(0)
      expect(mockOnConfirmDelete).toHaveBeenCalledOnce()
    })

    it('should clear timeout when delete dialog is closed', () => {
      const mockOnConfirmDelete = vi.fn()

      dialogManager.handleDeleteKeyPress(mockOnConfirmDelete)
      expect(dialogManager.deleteKeyPressCount).toBe(1)

      dialogManager.closeDeleteDialog()
      expect(dialogManager.deleteKeyPressCount).toBe(0)

      vi.advanceTimersByTime(2000)
      expect(dialogManager.deleteKeyPressCount).toBe(0)
    })
  })
})
