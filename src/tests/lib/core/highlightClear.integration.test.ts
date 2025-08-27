import { describe, it, expect, beforeEach, vi } from 'vitest'
import { mockInvoke, resetAllMocks } from '../../test-utils'

vi.mock('@tauri-apps/api/core', () => ({
  invoke: mockInvoke,
}))

describe('Highlight Clear Integration', () => {
  beforeEach(() => {
    resetAllMocks()
  })

  it('should re-enable highlights when user types after clearing with ESC', async () => {
    const { createSearchManager } = await import(
      '../../../lib/core/searchManager.svelte'
    )
    const { createAppCoordinator } = await import(
      '../../../lib/app/appCoordinator.svelte'
    )
    const { createEditorManager } = await import(
      '../../../lib/core/editorManager.svelte'
    )
    const { createFocusManager } = await import(
      '../../../lib/core/focusManager.svelte'
    )

    // Create manager instances for testing
    const { noteService } = await import(
      '../../../lib/services/noteService.svelte'
    )
    const searchManager = createSearchManager({ noteService })
    const editorManager = createEditorManager({ noteService })
    const focusManager = createFocusManager()
    const appCoordinator = createAppCoordinator({
      searchManager,
      editorManager,
      focusManager,
    })

    const { contentNavigationManager } = appCoordinator.managers

    searchManager.searchInput = 'test query'
    // areHighlightsCleared starts as false by default
    expect(contentNavigationManager.areHighlightsCleared).toBe(false)

    contentNavigationManager.clearHighlights()
    expect(contentNavigationManager.areHighlightsCleared).toBe(true)

    searchManager.searchInput = 'new search'

    await new Promise((resolve) => setTimeout(resolve, 0))

    // Highlights are re-enabled when new search happens
    // This behavior might need to be implemented in the search logic
    expect(contentNavigationManager.areHighlightsCleared).toBe(false)
  })
})
