/**
 * App Layer - Application Effects
 * Reactive side effects using Svelte 5 $effect runes.
 * Handles selection normalization, content loading, and highlight updates.
 */

interface AppEffectsDeps {
  getSelectedNote: () => string | null;
  getAreHighlightsCleared: () => boolean;
  focusManager: {
    selectedIndex: number;
    scrollToSelected: () => void;
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

export function setupAppEffects(deps: AppEffectsDeps): () => void {
  const {
    getSelectedNote,
    getAreHighlightsCleared,
    focusManager,
    contentManager,
    noteService,
    contentRequestController
  } = deps;


  // Async content loading function (pure)
  async function loadNoteContent(note: string, controller: AbortController): Promise<string> {
    try {
      const content = await noteService.getContent(note);
      return controller.signal.aborted ? '' : content;
    } catch (e) {
      if (!controller.signal.aborted) {
        console.error("Failed to load note content:", e);
      }
      return `Error loading note: ${e}`;
    }
  }

  $effect(() => {
    const note = getSelectedNote();

    // Cancel previous request
    if (contentRequestController.current) {
      contentRequestController.current.abort();
    }

    if (!note) {
      contentManager.setNoteContent('');
      return;
    }

    const controller = new AbortController();
    contentRequestController.set(controller);

    // Trigger async loading and update state when done
    loadNoteContent(note, controller).then(content => {
      if (!controller.signal.aborted) {
        contentManager.setNoteContent(content);
        requestAnimationFrame(() => {
          contentManager.scrollToFirstMatch();
        });
      }
    });
  });

  $effect(() => {
    contentManager.updateHighlighterState({
      areHighlightsCleared: getAreHighlightsCleared()
    });
  });

  $effect(() => {
    requestAnimationFrame(() => {
      focusManager.scrollToSelected();
    });
  });

  return function cleanup(): void {
    if (contentRequestController.current) {
      contentRequestController.current.abort();
      contentRequestController.set(null);
    }
  };
}
