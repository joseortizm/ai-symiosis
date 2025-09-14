/**
 * App Layer - Application Effects
 * Reactive side effects using Svelte 5 $effect runes.
 */

interface AppEffectsDeps {
  getHideHighlights: () => boolean
  focusManager: {
    selectedIndex: number
    scrollToSelectedInList: (index: number) => void
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
      focusManager.scrollToSelectedInList(selectedIndex)
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
