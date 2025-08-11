/**
 * App Layer - Application Effects
 * Reactive side effects using Svelte 5 $effect runes.
 * Handles selection normalization, content loading, and highlight updates.
 */

interface AppEffectsDeps {
  getFilteredNotes: () => string[];
  getSelectedIndex: () => number;
  getSelectedNote: () => string | null;
  getAreHighlightsCleared: () => boolean;
  setSelectedIndex: (index: number) => void;
  editorManager: {
    exitEditMode: () => void;
  };
  focusManager: {
    scrollToSelected: (index: number) => void;
  };
  contentManager: {
    setNoteContent: (content: string) => void;
    scrollToFirstMatch: () => void;
    updateHighlighterState: (state: { areHighlightsCleared: boolean }) => void;
  };
  noteService: {
    getContent: (noteName: string) => Promise<string>;
  };
  contentRequestController: {
    current: AbortController | null;
    set: (controller: AbortController | null) => void;
  };
}

export function setupAppEffects(deps: AppEffectsDeps) {
  const {
    getFilteredNotes,
    getSelectedIndex,
    getSelectedNote,
    getAreHighlightsCleared,
    setSelectedIndex,
    editorManager,
    focusManager,
    contentManager,
    noteService,
    contentRequestController
  } = deps;

  $effect(() => {
    const notes = getFilteredNotes();
    const currentIndex = getSelectedIndex();

    if (notes.length > 0 && (currentIndex === -1 || currentIndex >= notes.length)) {
      editorManager.exitEditMode();
    }
  });

  $effect(() => {
    const notes = getFilteredNotes();
    let index = getSelectedIndex();

    if (notes.length > 0) {
      if (index === -1 || index >= notes.length) {
        index = 0;
      }
      requestAnimationFrame(() => {
        focusManager.scrollToSelected(index);
      });
    }
  });

  $effect(() => {
    const note = getSelectedNote();

    if (!note) {
      contentManager.setNoteContent('');
      return;
    }

    if (contentRequestController.current) {
      contentRequestController.current.abort();
    }

    const controller = new AbortController();
    contentRequestController.set(controller);
    const currentController = controller;

    (async () => {
      try {
        const content = await noteService.getContent(note);

        if (!currentController.signal.aborted) {
          contentManager.setNoteContent(content);

          requestAnimationFrame(() => {
            contentManager.scrollToFirstMatch();
          });
        }
      } catch (e) {
        if (!currentController.signal.aborted) {
          console.error("Failed to load note content:", e);
          contentManager.setNoteContent(`Error loading note: ${e}`);
        }
      }
    })();
  });

  $effect(() => {
    contentManager.updateHighlighterState({
      areHighlightsCleared: getAreHighlightsCleared()
    });
  });

  return function cleanup() {
    if (contentRequestController.current) {
      contentRequestController.current.abort();
      contentRequestController.set(null);
    }
  };
}
