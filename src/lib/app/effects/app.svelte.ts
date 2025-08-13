/**
 * App Layer - Application Effects
 * Reactive side effects using Svelte 5 $effect runes.
 * Handles selection normalization, content loading, and highlight updates.
 */

interface AppEffectsDeps {
  getAreHighlightsCleared: () => boolean
  focusManager: {
    selectedIndex: number
    scrollToSelected: () => void
  }
  contentManager: {
    scrollToFirstMatch: () => void
  }
}

export function setupAppEffects(deps: AppEffectsDeps): () => void {
  const { getAreHighlightsCleared, focusManager, contentManager } = deps

  $effect(() => {
    requestAnimationFrame(() => {
      focusManager.scrollToSelected()
    })
  })

  return function cleanup(): void {}
}
