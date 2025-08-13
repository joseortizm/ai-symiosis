import { describe, it, expect, beforeEach, vi } from 'vitest'
import { mockInvoke, resetAllMocks } from '../../test-utils'
import type { EditorManager } from '../../../lib/core/editorManager.svelte'
import { createEditorManager } from '../../../lib/core/editorManager.svelte'

vi.mock('@tauri-apps/api/core', () => ({
  invoke: mockInvoke,
}))

// Create a fresh instance for each test
let editorManager: EditorManager

describe('editorManager', () => {
  beforeEach(() => {
    resetAllMocks()
    // Create fresh editor manager instance
    editorManager = createEditorManager()
  })

  describe('state getters', () => {
    it('should initialize with default state', () => {
      expect(editorManager.isEditMode).toBe(false)
      expect(editorManager.editContent).toBe('')
      expect(editorManager.isDirty).toBe(false)
      expect(editorManager.nearestHeaderText).toBe('')
    })
  })

  describe('enterEditMode', () => {
    it('should enter edit mode with raw content from API', async () => {
      const mockRawContent = '# Test Note\n\nThis is raw content'
      mockInvoke.mockResolvedValue(mockRawContent)

      await editorManager.enterEditMode('test-note.md')

      expect(mockInvoke).toHaveBeenCalledWith('get_note_raw_content', {
        noteName: 'test-note.md',
      })
      expect(editorManager.isEditMode).toBe(true)
      expect(editorManager.editContent).toBe(mockRawContent)
      expect(editorManager.isDirty).toBe(false)
    })

    it('should fallback to HTML content extraction if API fails', async () => {
      const mockHtmlContent = '<h1>Test Note</h1><p>This is HTML content</p>'
      mockInvoke.mockRejectedValue(new Error('API failed'))

      await editorManager.enterEditMode('test-note.md', mockHtmlContent)

      expect(editorManager.isEditMode).toBe(true)
      expect(editorManager.editContent).toBe(
        'Test Note\n\nThis is HTML content'
      )
    })

    it('should detect nearest header text when entering edit mode', async () => {
      const mockRawContent =
        '# Header 1\n\nSome content\n\n## Header 2\n\nMore content'
      mockInvoke.mockResolvedValue(mockRawContent)

      // Mock DOM element with getBoundingClientRect and querySelectorAll
      const mockElement: Partial<HTMLElement> = {
        getBoundingClientRect: vi
          .fn()
          .mockReturnValue({ top: 100, height: 600 }),
        querySelectorAll: vi.fn().mockReturnValue([
          {
            getBoundingClientRect: vi.fn().mockReturnValue({ top: 120 }),
            textContent: 'Header 2',
          },
        ] as any),
      }

      await editorManager.enterEditMode(
        'test-note.md',
        '',
        mockElement as HTMLElement
      )

      expect(editorManager.nearestHeaderText).toBe('Header 2')
    })

    it('should handle missing note name', async () => {
      await editorManager.enterEditMode('')

      expect(editorManager.isEditMode).toBe(false)
      expect(mockInvoke).not.toHaveBeenCalled()
    })
  })

  describe('exitEditMode', () => {
    it('should exit edit mode and clear state', async () => {
      // First enter edit mode
      mockInvoke.mockResolvedValue('test content')
      await editorManager.enterEditMode('test-note.md')

      // Then exit
      editorManager.exitEditMode()

      expect(editorManager.isEditMode).toBe(false)
      expect(editorManager.editContent).toBe('')
      expect(editorManager.isDirty).toBe(false)
      expect(editorManager.nearestHeaderText).toBe('')
    })
  })

  describe('updateContent', () => {
    beforeEach(async () => {
      mockInvoke.mockResolvedValue('original content')
      await editorManager.enterEditMode('test-note.md')
    })

    it('should update content and mark as dirty', () => {
      editorManager.updateContent('modified content')

      expect(editorManager.editContent).toBe('modified content')
      expect(editorManager.isDirty).toBe(true)
    })

    it('should not mark as dirty if content matches original', () => {
      editorManager.updateContent('original content')

      expect(editorManager.editContent).toBe('original content')
      expect(editorManager.isDirty).toBe(false)
    })
  })

  describe('saveNote', () => {
    const mockNoteName = 'test-note.md'

    it('should save note content via API', async () => {
      // Setup: enter edit mode first
      mockInvoke.mockResolvedValue('original content')
      await editorManager.enterEditMode(mockNoteName)
      editorManager.updateContent('modified content')

      // Test: save the note
      mockInvoke.mockResolvedValue(undefined)

      const result = await editorManager.saveNote(mockNoteName)

      expect(mockInvoke).toHaveBeenCalledWith('save_note', {
        noteName: mockNoteName,
        content: 'modified content',
      })
      expect(result.success).toBe(true)
      expect(editorManager.isDirty).toBe(false)
    })

    it('should return error if save fails', async () => {
      // Setup: enter edit mode first
      mockInvoke.mockResolvedValue('original content')
      await editorManager.enterEditMode(mockNoteName)
      editorManager.updateContent('modified content')

      // Test: simulate save failure
      const error = new Error('Save failed')
      mockInvoke.mockRejectedValue(error)

      const result = await editorManager.saveNote(mockNoteName)

      expect(result.success).toBe(false)
      expect(result.error).toBe('Save failed')
      expect(editorManager.isDirty).toBe(true) // Should remain dirty on failure
    })

    it('should handle empty note name', async () => {
      const result = await editorManager.saveNote('')

      expect(result.success).toBe(false)
      expect(result.error).toBe('No note selected')
      expect(mockInvoke).not.toHaveBeenCalled()
    })

    it('should handle empty content', async () => {
      // Setup: enter edit mode first
      mockInvoke.mockResolvedValue('original content')
      await editorManager.enterEditMode(mockNoteName)
      editorManager.updateContent('')

      // Test: save with empty content
      mockInvoke.mockResolvedValue(undefined)
      const result = await editorManager.saveNote(mockNoteName)

      expect(mockInvoke).toHaveBeenCalledWith('save_note', {
        noteName: mockNoteName,
        content: '',
      })
      expect(result.success).toBe(true)
    })
  })

  describe('showExitEditDialog integration', () => {
    it('should work with dirty state', async () => {
      mockInvoke.mockResolvedValue('content')
      await editorManager.enterEditMode('test.md')
      editorManager.updateContent('modified')

      expect(editorManager.isDirty).toBe(true)
      expect(editorManager.isEditMode).toBe(true)
    })
  })
})
