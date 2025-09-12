/**
 * Core Layer - Content Highlighting
 * Search term highlighting functionality with caching for performance.
 * Handles regex escaping, term extraction, and HTML content highlighting.
 */

interface CacheEntry {
  result: string
  timestamp: number
  accessCount: number
}

const MAX_CACHE_SIZE = 100
const CACHE_TTL = 5 * 60 * 1000 // 5 minutes
const highlightCache = new Map<string, CacheEntry>()

function escapeRegex(text: string): string {
  return text.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
}

function cleanExpiredEntries(): void {
  const now = Date.now()
  const expiredKeys = findExpiredCacheKeys(now)
  removeExpiredKeys(expiredKeys)
}

function findExpiredCacheKeys(currentTime: number): string[] {
  const expiredKeys: string[] = []

  for (const [key, entry] of highlightCache) {
    if (isCacheEntryExpired(entry, currentTime)) {
      expiredKeys.push(key)
    }
  }

  return expiredKeys
}

function isCacheEntryExpired(entry: CacheEntry, currentTime: number): boolean {
  return currentTime - entry.timestamp > CACHE_TTL
}

function removeExpiredKeys(keys: string[]): void {
  for (const key of keys) {
    highlightCache.delete(key)
  }
}

function evictLRUEntry(): void {
  const lruKey = findLeastRecentlyUsedKey()
  if (lruKey) {
    highlightCache.delete(lruKey)
  }
}

function findLeastRecentlyUsedKey(): string | null {
  let oldestKey: string | null = null
  let oldestAccess = Infinity

  for (const [key, entry] of highlightCache) {
    if (isLessAccessedThanCurrent(entry, oldestAccess)) {
      oldestAccess = entry.accessCount
      oldestKey = key
    }
  }

  return oldestKey
}

function isLessAccessedThanCurrent(
  entry: CacheEntry,
  currentMinAccess: number
): boolean {
  return entry.accessCount < currentMinAccess
}

function highlightMatches(content: string, query: string): string {
  if (!query.trim()) {
    return content
  }

  const cached = getCachedHighlight(content, query)
  if (cached) return cached

  return generateAndCacheHighlight(content, query)
}

function getCachedHighlight(content: string, query: string): string | null {
  const key = generateCacheKey(content, query)
  const cached = highlightCache.get(key)

  if (cached) {
    updateCacheAccess(cached)
    return cached.result
  }

  return null
}

function generateCacheKey(content: string, query: string): string {
  return `${content.substring(0, 100)}:${query}`
}

function updateCacheAccess(entry: CacheEntry): void {
  entry.accessCount++
  entry.timestamp = Date.now()
}

function generateAndCacheHighlight(content: string, query: string): string {
  const result = performHighlighting(content, query)
  cacheHighlightResult(content, query, result)
  return result
}

function performHighlighting(content: string, query: string): string {
  const escapedQuery = escapeRegex(query)
  const regex = new RegExp(`(${escapedQuery})`, 'gi')
  return content.replace(regex, '<mark class="highlight">$1</mark>')
}

function cacheHighlightResult(
  content: string,
  query: string,
  result: string
): void {
  maintainCacheSize()
  const key = generateCacheKey(content, query)

  highlightCache.set(key, {
    result,
    timestamp: Date.now(),
    accessCount: 1,
  })
}

function maintainCacheSize(): void {
  cleanExpiredEntries()

  if (highlightCache.size >= MAX_CACHE_SIZE) {
    evictLRUEntry()
  }
}

export function getHighlightedContent(
  content: string,
  query: string,
  hideHighlights: boolean
): string {
  if (!query.trim() || hideHighlights) {
    return content
  }
  return highlightMatches(content, query)
}

export function getHighlightedTitle(
  title: string,
  query: string,
  hideHighlights: boolean = false
): string {
  if (!query.trim() || query.length < 3 || hideHighlights) {
    return title
  }
  return highlightMatches(title, query)
}

export function clearHighlightCache(): void {
  highlightCache.clear()
}

// Periodic cleanup every 30 seconds
if (typeof window !== 'undefined') {
  setInterval(cleanExpiredEntries, 30000)
}
