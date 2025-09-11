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
  editingNoteName: string | null
  exitHeaderText: string
}

interface SaveResult {
  success: boolean
  error?: string
}

interface EditorManagerDeps {
  noteService: ReturnType<typeof createNoteService>
  contentNavigationManager: {
    getCurrentHeaderText(): string
  }
}

export interface EditorManager {
  readonly isEditMode: boolean
  readonly editContent: string
  readonly isDirty: boolean
  readonly nearestHeaderText: string
  readonly editingNoteName: string | null
  enterEditMode(
    noteName: string,
    fallbackHtmlContent?: string,
    noteContentElement?: HTMLElement
  ): Promise<void>
  exitEditMode(): string
  updateContent(newContent: string): void
  saveNote(): Promise<SaveResult>
  setExitHeaderText(headerText: string): void
}

export function createEditorManager(deps: EditorManagerDeps): EditorManager {
  const state = $state<EditorState>({
    isEditMode: false,
    editContent: '',
    originalContent: '',
    nearestHeaderText: '',
    editingNoteName: null,
    exitHeaderText: '',
  })

  async function enterEditMode(
    noteName: string,
    fallbackHtmlContent?: string,
    _noteContentElement?: HTMLElement
  ): Promise<void> {
    if (!noteName) {
      return
    }

    state.nearestHeaderText =
      deps.contentNavigationManager.getCurrentHeaderText()
    await loadEditContent(noteName, fallbackHtmlContent)
  }

  async function loadEditContent(
    noteName: string,
    fallbackHtmlContent?: string
  ): Promise<void> {
    try {
      const rawContent = await deps.noteService.getRawContent(noteName)
      setEditState(rawContent, noteName)
    } catch (e) {
      console.error('Failed to load raw note content:', e)

      if (fallbackHtmlContent) {
        const extractedContent = convertHtmlToText(fallbackHtmlContent)
        setEditState(extractedContent, noteName)
      }
    }
  }

  function convertHtmlToText(htmlContent: string): string {
    const tempDiv = document.createElement('div')
    tempDiv.innerHTML = htmlContent

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

    return extractedContent.replace(/\n\n+/g, '\n\n').trim()
  }

  function setEditState(content: string, noteName: string): void {
    state.isEditMode = true
    state.editContent = content
    state.originalContent = content
    state.editingNoteName = noteName
  }

  function exitEditMode(): string {
    const exitHeader = state.exitHeaderText
    state.isEditMode = false
    state.editContent = ''
    state.originalContent = ''
    state.nearestHeaderText = ''
    state.editingNoteName = null
    state.exitHeaderText = ''
    return exitHeader
  }

  function setExitHeaderText(headerText: string): void {
    state.exitHeaderText = headerText
  }

  function updateContent(newContent: string): void {
    state.editContent = newContent
  }

  async function saveNote(): Promise<SaveResult> {
    if (!state.editingNoteName) {
      return {
        success: false,
        error: 'No note being edited',
      }
    }

    try {
      await deps.noteService.save(
        state.editingNoteName,
        state.editContent,
        state.originalContent
      )

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

    get editingNoteName() {
      return state.editingNoteName
    },

    enterEditMode,
    exitEditMode,
    updateContent,
    saveNote,
    setExitHeaderText,
  }
}
