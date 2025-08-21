/**
 * Core Layer - Editor Manager
 * Note editing state including edit mode, content changes, and save operations.
 * Handles raw content loading, dirty state tracking, and nearest header detection.
 */

import type { createNoteService } from '../services/noteService.svelte'

interface EditorState {
  isEditMode: boolean
  editContent: string
  originalContent: string
  nearestHeaderText: string
}

interface SaveResult {
  success: boolean
  error?: string
}

interface EditorManagerDeps {
  noteService: ReturnType<typeof createNoteService>
}

export interface EditorManager {
  readonly isEditMode: boolean
  readonly editContent: string
  readonly isDirty: boolean
  readonly nearestHeaderText: string
  enterEditMode(
    noteName: string,
    fallbackHtmlContent?: string,
    noteContentElement?: HTMLElement
  ): Promise<void>
  exitEditMode(): void
  updateContent(newContent: string): void
  saveNote(noteName: string): Promise<SaveResult>
}

export function createEditorManager(deps: EditorManagerDeps): EditorManager {
  const state = $state<EditorState>({
    isEditMode: false,
    editContent: '',
    originalContent: '',
    nearestHeaderText: '',
  })

  async function enterEditMode(
    noteName: string,
    fallbackHtmlContent?: string,
    noteContentElement?: HTMLElement
  ): Promise<void> {
    if (!noteName) {
      return
    }

    detectAndSetNearestHeader(noteContentElement)
    await loadEditContent(noteName, fallbackHtmlContent)
  }

  function detectAndSetNearestHeader(noteContentElement?: HTMLElement): void {
    if (!noteContentElement) {
      return
    }

    try {
      const rect = noteContentElement.getBoundingClientRect()
      const headers = noteContentElement.querySelectorAll(
        'h1, h2, h3, h4, h5, h6'
      )
      let bestHeader: Element | null = null

      for (const header of headers) {
        const headerRect = header.getBoundingClientRect()

        // Check if header is in the viewport
        const isInViewport =
          headerRect.top >= rect.top &&
          headerRect.top <= rect.top + (rect.height || 600)

        if (isInViewport) {
          // If header is visible, use it (prefer the first visible header)
          if (!bestHeader) {
            bestHeader = header
          }
        } else if (headerRect.top < rect.top) {
          // If header is above viewport, keep track of it as the "last passed" header
          bestHeader = header
        }
      }

      if (bestHeader) {
        state.nearestHeaderText = bestHeader.textContent?.trim() || ''
      }
    } catch (e) {
      console.warn('Failed to detect nearest header:', e)
    }
  }

  async function loadEditContent(
    noteName: string,
    fallbackHtmlContent?: string
  ): Promise<void> {
    try {
      const rawContent = await deps.noteService.getRawContent(noteName)
      setEditState(rawContent)
    } catch (e) {
      console.error('Failed to load raw note content:', e)

      if (fallbackHtmlContent) {
        const extractedContent = convertHtmlToText(fallbackHtmlContent)
        setEditState(extractedContent)
      }
    }
  }

  function convertHtmlToText(htmlContent: string): string {
    const tempDiv = document.createElement('div')
    tempDiv.innerHTML = htmlContent

    // Convert HTML to text with proper line breaks
    let extractedContent = ''
    const walker = document.createTreeWalker(
      tempDiv,
      NodeFilter.SHOW_TEXT | NodeFilter.SHOW_ELEMENT,
      null
    )

    let node
    while ((node = walker.nextNode())) {
      if (node.nodeType === Node.TEXT_NODE) {
        extractedContent += node.textContent
      } else if (node.nodeType === Node.ELEMENT_NODE) {
        const tagName = (node as Element).tagName.toLowerCase()
        if (
          ['p', 'div', 'br', 'h1', 'h2', 'h3', 'h4', 'h5', 'h6'].includes(
            tagName
          )
        ) {
          extractedContent += '\n\n'
        }
      }
    }

    // Clean up extra whitespace and normalize line breaks
    return extractedContent.replace(/\n\n+/g, '\n\n').trim()
  }

  function setEditState(content: string): void {
    state.isEditMode = true
    state.editContent = content
    state.originalContent = content
  }

  function exitEditMode(): void {
    state.isEditMode = false
    state.editContent = ''
    state.originalContent = ''
    state.nearestHeaderText = ''
  }

  function updateContent(newContent: string): void {
    state.editContent = newContent
  }

  async function saveNote(noteName: string): Promise<SaveResult> {
    if (!noteName) {
      return {
        success: false,
        error: 'No note selected',
      }
    }

    try {
      await deps.noteService.save(noteName, state.editContent)

      // Update original content to new saved content
      state.originalContent = state.editContent

      return { success: true }
    } catch (e) {
      const errorMessage = e instanceof Error ? e.message : 'Save failed'
      console.error('Failed to save note:', e)

      return {
        success: false,
        error: errorMessage,
      }
    }
  }

  return {
    // Reactive getters
    get isEditMode() {
      return state.isEditMode
    },

    get editContent() {
      return state.editContent
    },

    get isDirty() {
      return state.editContent !== state.originalContent
    },

    get nearestHeaderText() {
      return state.nearestHeaderText
    },

    // Actions
    enterEditMode,
    exitEditMode,
    updateContent,
    saveNote,
  }
}
