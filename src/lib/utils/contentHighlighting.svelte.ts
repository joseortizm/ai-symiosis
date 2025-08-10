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
  setContent(content: string): void {
    state.content = content;
  },

  setQuery(query: string): void {
    externalQuery = query;
  },

  updateHighlighterState(newState: { query?: string; areHighlightsCleared?: boolean }): void {
    if (newState.query !== undefined) {
      externalQuery = newState.query;
    }
    if (newState.areHighlightsCleared !== undefined) {
      externalAreHighlightsCleared = newState.areHighlightsCleared;
    }
  },

  get highlighted(): string {
    return highlighted();
  },

  get content(): string {
    return state.content;
  },

  get query(): string {
    return externalQuery;
  },

  get areHighlightsCleared(): boolean {
    return externalAreHighlightsCleared;
  },

  set areHighlightsCleared(value: boolean) {
    externalAreHighlightsCleared = value;
  },

  clearCache(): void {
    highlightCache.clear();
  }
};
