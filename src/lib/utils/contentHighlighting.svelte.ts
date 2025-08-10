interface HighlighterState {
  content: string;
  query: string;
  areHighlightsCleared: boolean;
}

const state = $state<HighlighterState>({
  content: '',
  query: '',
  areHighlightsCleared: false
});

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
  if (!state.query.trim() || state.areHighlightsCleared) {
    return state.content;
  }
  return highlightMatches(state.content, state.query);
});

export const contentHighlighter = {
  setContent(content: string): void {
    state.content = content;
  },

  setQuery(query: string): void {
    state.query = query;
  },

  updateHighlighterState(newState: { query?: string; areHighlightsCleared?: boolean }): void {
    if (newState.query !== undefined) {
      state.query = newState.query;
    }
    if (newState.areHighlightsCleared !== undefined) {
      state.areHighlightsCleared = newState.areHighlightsCleared;
    }
  },

  get highlighted(): string {
    return highlighted();
  },

  get content(): string {
    return state.content;
  },

  get query(): string {
    return state.query;
  },

  get areHighlightsCleared(): boolean {
    return state.areHighlightsCleared;
  },

  set areHighlightsCleared(value: boolean) {
    state.areHighlightsCleared = value;
  },

  clearCache(): void {
    highlightCache.clear();
  }
};
