import { describe, it, expect, beforeEach, vi } from 'vitest'
import { mockInvoke, resetAllMocks } from '../../test-utils'

vi.mock('@tauri-apps/api/core', () => ({
  invoke: mockInvoke,
}))

// Import after mocking
const { createEditorManager } = await import(
  '../../../lib/core/editorManager.svelte'
)

// Create a fresh instance for each test
let editorManager: ReturnType<typeof createEditorManager>

describe('editorManager integration', () => {
  beforeEach(async () => {
    resetAllMocks()
    // Use real noteService for integration tests
    const { noteService } = await import(
      '../../../lib/services/noteService.svelte'
    )

    const mockContentNavigationManager = {
      getCurrentHeaderText: vi.fn().mockReturnValue(''),
    }

    editorManager = createEditorManager({
      noteService,
      contentNavigationManager: mockContentNavigationManager,
    })
  })

  describe('real API compatibility', () => {
    it('should call correct save_note_with_content_check API endpoint', async () => {
      // Setup: enter edit mode with a note
      mockInvoke.mockResolvedValue('test content')
      await editorManager.enterEditMode('test.md')
      editorManager.updateContent('modified content')

      // Test: save should call the correct API
      mockInvoke.mockResolvedValue(undefined)
      const result = await editorManager.saveNote()

      // Assert: should call 'save_note_with_content_check' with original content validation
      expect(mockInvoke).toHaveBeenCalledWith('save_note_with_content_check', {
        noteName: 'test.md',
        content: 'modified content',
        originalContent: 'test content',
      })
      expect(result.success).toBe(true)
    })

    it('should match noteService API call format', async () => {
      // This test ensures editorManager uses same API as noteService
      const { noteService } = await import(
        '../../../lib/services/noteService.svelte'
      )

      // Setup editorManager
      mockInvoke.mockResolvedValue('test content')
      await editorManager.enterEditMode('test.md')
      editorManager.updateContent('modified content')

      // Reset mocks to track calls
      mockInvoke.mockClear()
      mockInvoke.mockResolvedValue(undefined)

      // Call both save methods
      await editorManager.saveNote()
      await noteService.save('test.md', 'modified content', 'test content')

      // Both should call the same API with same parameters
      const editorCall = mockInvoke.mock.calls[0]
      const serviceCall = mockInvoke.mock.calls[1]

      expect(editorCall[0]).toBe(serviceCall[0]) // Same API endpoint
      expect(editorCall[1]).toEqual(serviceCall[1]) // Same parameters
    })

    it('should handle missing content gracefully like real save', async () => {
      // Setup: empty content case
      mockInvoke.mockResolvedValue('')
      await editorManager.enterEditMode('empty.md')
      // editorManager.updateContent(''); // Keep empty

      mockInvoke.mockResolvedValue(undefined)
      const result = await editorManager.saveNote()

      expect(mockInvoke).toHaveBeenCalledWith('save_note_with_content_check', {
        noteName: 'empty.md',
        content: '',
        originalContent: '',
      })
      expect(result.success).toBe(true)
    })
  })

  describe('editor workflow integration', () => {
    it('should save but NOT exit editor mode (save only)', async () => {
      // Setup: enter edit mode
      mockInvoke.mockResolvedValue('test content')
      await editorManager.enterEditMode('test.md')
      editorManager.updateContent('modified content')

      expect(editorManager.isEditMode).toBe(true)
      expect(editorManager.isDirty).toBe(true)

      // Test: save should complete but NOT exit edit mode
      mockInvoke.mockResolvedValue(undefined)
      const result = await editorManager.saveNote()

      expect(result.success).toBe(true)
      expect(editorManager.isDirty).toBe(false)

      // CORRECT: saveNote alone should NOT exit edit mode
      expect(editorManager.isEditMode).toBe(true)
    })

    it('should allow saving and exiting separately for clean architecture', async () => {
      // Setup: enter edit mode
      mockInvoke.mockResolvedValue('test content')
      await editorManager.enterEditMode('test.md')
      editorManager.updateContent('modified content')

      // Test: save should work without exiting edit mode
      mockInvoke.mockResolvedValue(undefined)

      const result = await editorManager.saveNote()

      expect(result.success).toBe(true)
      expect(editorManager.isEditMode).toBe(true) // Still in edit mode after save
      expect(editorManager.isDirty).toBe(false) // But not dirty anymore

      // And exit should be separate
      editorManager.exitEditMode()
      expect(editorManager.isEditMode).toBe(false)
    })
  })
})
