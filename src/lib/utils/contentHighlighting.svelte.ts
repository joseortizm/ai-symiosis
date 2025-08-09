interface HighlighterState {
  content: string;
}

const state = $state<HighlighterState>({
  content: ''
});

let externalQuery = $state('');
let externalAreHighlightsCleared = $state(false);

const highlightCache = new Map<string, string>();

function escapeRegex(text: string): string {
  return text.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
}

function highlightMatches(content: string, query: string): string {
  if (!query.trim()) {
    return content;
  }

  const key = `${content.substring(0, 100)}:${query}`;
  if (highlightCache.has(key)) {
    return highlightCache.get(key)!;
  }

  const escapedQuery = escapeRegex(query);
  const regex = new RegExp(`(${escapedQuery})`, 'gi');
  const result = content.replace(regex, '<mark class="highlight">$1</mark>');

  if (highlightCache.size > 50) {
    const firstKey = highlightCache.keys().next().value!;
    highlightCache.delete(firstKey);
  }
  highlightCache.set(key, result);

  return result;
}

const highlighted = $derived(() => {
  if (!externalQuery.trim() || externalAreHighlightsCleared) {
    return state.content;
  }
  return highlightMatches(state.content, externalQuery);
});

export const contentHighlighter = {
  updateState(newState: { content?: string; query?: string; areHighlightsCleared?: boolean }): void {
    if (newState.content !== undefined) {
      state.content = newState.content;
    }
    if (newState.query !== undefined) {
      externalQuery = newState.query;
    }
    if (newState.areHighlightsCleared !== undefined) {
      externalAreHighlightsCleared = newState.areHighlightsCleared;
    }
  },

  setExternalState(query: string, areHighlightsCleared: boolean): void {
    externalQuery = query;
    externalAreHighlightsCleared = areHighlightsCleared;
  },

  get highlighted(): string {
    return highlighted();
  },

  clearCache(): void {
    highlightCache.clear();
  }
};
