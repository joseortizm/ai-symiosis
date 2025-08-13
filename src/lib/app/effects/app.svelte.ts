/**
 * App Layer - Application Effects
 * Reactive side effects using Svelte 5 $effect runes.
 * Handles selection normalization, content loading, and highlight updates.
 */

interface AppEffectsDeps {
  getAreHighlightsCleared: () => boolean
  focusManager: {
    selectedIndex: number
    scrollToIndex: (index: number) => void
  }
  contentManager: {
    scrollToFirstMatch: () => void
  }
}

export function setupAppEffects(deps: AppEffectsDeps): () => void {
  const { focusManager } = deps

  $effect(() => {
    const selectedIndex = focusManager.selectedIndex
    requestAnimationFrame(() => {
      focusManager.scrollToIndex(selectedIndex)
    })
  })

  return function cleanup(): void {}
}
