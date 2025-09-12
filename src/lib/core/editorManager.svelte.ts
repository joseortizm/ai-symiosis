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
    if (!validateNoteNameForEdit(noteName)) {
      return
    }

    prepareEditModeState()
    await loadEditContent(noteName, fallbackHtmlContent)
  }

  function validateNoteNameForEdit(noteName: string): boolean {
    return Boolean(noteName)
  }

  function prepareEditModeState(): void {
    state.nearestHeaderText =
      deps.contentNavigationManager.getCurrentHeaderText()
  }

  async function loadEditContent(
    noteName: string,
    fallbackHtmlContent?: string
  ): Promise<void> {
    try {
      await loadRawContentForEdit(noteName)
    } catch (e) {
      console.error('Failed to load raw note content:', e)
      handleRawContentLoadFailure(noteName, fallbackHtmlContent)
    }
  }

  async function loadRawContentForEdit(noteName: string): Promise<void> {
    const rawContent = await deps.noteService.getRawContent(noteName)
    setEditState(rawContent, noteName)
  }

  function handleRawContentLoadFailure(
    noteName: string,
    fallbackHtmlContent?: string
  ): void {
    if (fallbackHtmlContent) {
      const extractedContent = convertHtmlToText(fallbackHtmlContent)
      setEditState(extractedContent, noteName)
    }
  }

  function convertHtmlToText(htmlContent: string): string {
    const tempDiv = createHtmlContainer(htmlContent)
    const extractedContent = extractTextFromHtmlTree(tempDiv)
    return cleanupExtractedText(extractedContent)
  }

  function createHtmlContainer(htmlContent: string): HTMLDivElement {
    const tempDiv = document.createElement('div')
    tempDiv.innerHTML = htmlContent
    return tempDiv
  }

  function extractTextFromHtmlTree(container: HTMLDivElement): string {
    const walker = document.createTreeWalker(
      container,
      NodeFilter.SHOW_TEXT | NodeFilter.SHOW_ELEMENT,
      null
    )

    let extractedContent = ''
    let node

    while ((node = walker.nextNode())) {
      if (node.nodeType === Node.TEXT_NODE) {
        extractedContent += node.textContent
      } else if (node.nodeType === Node.ELEMENT_NODE) {
        extractedContent += processElementNode(node as Element)
      }
    }

    return extractedContent
  }

  function processElementNode(element: Element): string {
    const tagName = element.tagName.toLowerCase()
    const blockLevelTags = [
      'p',
      'div',
      'br',
      'h1',
      'h2',
      'h3',
      'h4',
      'h5',
      'h6',
    ]

    return blockLevelTags.includes(tagName) ? '\n\n' : ''
  }

  function cleanupExtractedText(content: string): string {
    return content.replace(/\n\n+/g, '\n\n').trim()
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
