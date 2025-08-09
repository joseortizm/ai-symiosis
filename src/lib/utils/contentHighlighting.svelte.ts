interface HighlighterState {
  query: string;
  content: string;
  areHighlightsCleared: boolean;
}

const state = $state<HighlighterState>({
  query: '',
  content: '',
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
  updateState(newState: Partial<HighlighterState>): void {
    Object.assign(state, newState);
  },

  get highlighted(): string {
    return highlighted();
  },

  clearCache(): void {
    highlightCache.clear();
  }
};
