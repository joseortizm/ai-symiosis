import { invoke } from '@tauri-apps/api/core';

interface EditorState {
  isEditMode: boolean;
  editContent: string;
  originalContent: string;
  nearestHeaderText: string;
}

const state = $state<EditorState>({
  isEditMode: false,
  editContent: '',
  originalContent: '',
  nearestHeaderText: ''
});

interface SaveResult {
  success: boolean;
  error?: string;
}

export const editorManager = {
  // Reactive getters
  get isEditMode() {
    return state.isEditMode;
  },

  get editContent() {
    return state.editContent;
  },

  get isDirty() {
    return state.editContent !== state.originalContent;
  },

  get nearestHeaderText() {
    return state.nearestHeaderText;
  },

  // Actions
  async enterEditMode(noteName: string, fallbackHtmlContent?: string, noteContentElement?: HTMLElement): Promise<void> {
    if (!noteName) {
      return;
    }

    // Detect nearest header if element provided
    if (noteContentElement) {
      try {
        const rect = noteContentElement.getBoundingClientRect();
        const headers = noteContentElement.querySelectorAll('h1, h2, h3, h4, h5, h6');

        for (const header of headers) {
          const headerRect = header.getBoundingClientRect();
          if (headerRect.top >= rect.top) {
            state.nearestHeaderText = header.textContent?.trim() || '';
            break;
          }
        }
      } catch (e) {
        console.warn('Failed to detect nearest header:', e);
      }
    }

    try {
      const rawContent = await invoke<string>("get_note_raw_content", { noteName });
      state.isEditMode = true;
      state.editContent = rawContent;
      state.originalContent = rawContent;
    } catch (e) {
      console.error("Failed to load raw note content:", e);

      // Fallback: extract text from HTML content
      if (fallbackHtmlContent) {
        const tempDiv = document.createElement('div');
        tempDiv.innerHTML = fallbackHtmlContent;

        // Convert HTML to text with proper line breaks
        let extractedContent = '';
        const walker = document.createTreeWalker(
          tempDiv,
          NodeFilter.SHOW_TEXT | NodeFilter.SHOW_ELEMENT,
          null
        );

        let node;
        while (node = walker.nextNode()) {
          if (node.nodeType === Node.TEXT_NODE) {
            extractedContent += node.textContent;
          } else if (node.nodeType === Node.ELEMENT_NODE) {
            const tagName = (node as Element).tagName.toLowerCase();
            if (['p', 'div', 'br', 'h1', 'h2', 'h3', 'h4', 'h5', 'h6'].includes(tagName)) {
              extractedContent += '\n\n';
            }
          }
        }

        // Clean up extra whitespace and normalize line breaks
        extractedContent = extractedContent.replace(/\n\n+/g, '\n\n').trim();

        state.isEditMode = true;
        state.editContent = extractedContent;
        state.originalContent = extractedContent;
      }
    }
  },

  exitEditMode(): void {
    state.isEditMode = false;
    state.editContent = '';
    state.originalContent = '';
    state.nearestHeaderText = '';
  },

  updateContent(newContent: string): void {
    state.editContent = newContent;
  },

  async saveNote(noteName: string): Promise<SaveResult> {
    if (!noteName) {
      return {
        success: false,
        error: 'No note selected'
      };
    }

    try {
      await invoke("save_note", {
        noteName,
        content: state.editContent
      });

      // Update original content to new saved content
      state.originalContent = state.editContent;

      return { success: true };
    } catch (e) {
      const errorMessage = e instanceof Error ? e.message : 'Save failed';
      console.error("Failed to save note:", e);

      return {
        success: false,
        error: errorMessage
      };
    }
  },

  async saveAndExit(noteName: string): Promise<SaveResult> {
    const result = await this.saveNote(noteName);

    if (result.success) {
      this.exitEditMode();
    }

    return result;
  }
};
