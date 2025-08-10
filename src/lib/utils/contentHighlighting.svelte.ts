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

export function getHighlightedContent(content: string, query: string, areHighlightsCleared: boolean): string {
  if (!query.trim() || areHighlightsCleared) {
    return content;
  }
  return highlightMatches(content, query);
}

export function clearHighlightCache(): void {
  highlightCache.clear();
}
