/**
 * App Layer - Application Effects
 * Reactive side effects using Svelte 5 $effect runes.
 * Handles selection normalization, content loading, and highlight updates.
 */

interface AppEffectsDeps {
  getHideHighlights: () => boolean
  focusManager: {
    selectedIndex: number
    scrollToIndex: (index: number) => void
  }
  contentManager: {
    scrollToFirstMatch: () => void
  }
  searchManager: {
    readonly query: string
  }
  contentNavigationManager: {
    showHighlights(): void
  }
}

export function setupAppEffects(deps: AppEffectsDeps): () => void {
  const { focusManager, searchManager, contentNavigationManager } = deps

  $effect(() => {
    const selectedIndex = focusManager.selectedIndex
    requestAnimationFrame(() => {
      focusManager.scrollToIndex(selectedIndex)
    })
  })

  // Reactive auto-reset: when user types new search, re-enable highlights
  $effect(() => {
    const query = searchManager.query
    if (query.trim() !== '') {
      contentNavigationManager.showHighlights()
    }
  })

  return function cleanup(): void {}
}
