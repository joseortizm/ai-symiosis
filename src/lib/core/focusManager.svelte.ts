/**
 * Core Layer - Focus Manager
 * Keyboard focus state and DOM element focus operations.
 * Tracks focused elements and provides programmatic focus control.
 */

interface FocusState {
  isSearchInputFocused: boolean;
  isNoteContentFocused: boolean;
  searchElement: HTMLInputElement | null;
  noteContentElement: HTMLElement | null;
  noteListElement: HTMLElement | null;
}

interface FocusManager {
  readonly isSearchInputFocused: boolean;
  readonly isNoteContentFocused: boolean;
  readonly searchElement: HTMLInputElement | null;
  readonly noteContentElement: HTMLElement | null;
  readonly noteListElement: HTMLElement | null;
  setSearchInputFocused(value: boolean): void;
  setNoteContentFocused(value: boolean): void;
  setSearchElement(element: HTMLInputElement | null): void;
  setNoteContentElement(element: HTMLElement | null): void;
  setNoteListElement(element: HTMLElement | null): void;
  focusSearch(): void;
  scrollNoteContentUp(): void;
  scrollNoteContentDown(): void;
  scrollToSelectedInList(selectedIndex: number): void;
  scrollToSelected(selectedIndex: number): void;
}

export function createFocusManager(): FocusManager {
  const state = $state<FocusState>({
    isSearchInputFocused: false,
    isNoteContentFocused: false,
    searchElement: null,
    noteContentElement: null,
    noteListElement: null
  });

  function scrollToSelectedInList(selectedIndex: number): void {
    if (state.noteListElement && selectedIndex >= 0) {
      const selectedButton = state.noteListElement.children[selectedIndex]?.querySelector('button');
      if (selectedButton) {
        selectedButton.scrollIntoView({ behavior: 'smooth', block: 'nearest' });
      }
    }
  }

  function scrollToSelected(selectedIndex: number): void {
    scrollToSelectedInList(selectedIndex);
  }

  return {
    // Reactive getters
    get isSearchInputFocused(): boolean {
      return state.isSearchInputFocused;
    },

    get isNoteContentFocused(): boolean {
      return state.isNoteContentFocused;
    },

    get searchElement(): HTMLInputElement | null {
      return state.searchElement;
    },

    get noteContentElement(): HTMLElement | null {
      return state.noteContentElement;
    },

    get noteListElement(): HTMLElement | null {
      return state.noteListElement;
    },

    // Focus state setters
    setSearchInputFocused(value: boolean): void {
      state.isSearchInputFocused = value;
    },

    setNoteContentFocused(value: boolean): void {
      state.isNoteContentFocused = value;
    },

    // Element setters
    setSearchElement(element: HTMLInputElement | null): void {
      state.searchElement = element;
    },

    setNoteContentElement(element: HTMLElement | null): void {
      state.noteContentElement = element;
    },

    setNoteListElement(element: HTMLElement | null): void {
      state.noteListElement = element;
    },

    // Focus actions
    focusSearch(): void {
      state.searchElement?.focus();
    },

    // Scroll actions
    scrollNoteContentUp(): void {
      state.noteContentElement?.scrollBy({
        top: -50,
        behavior: 'smooth'
      });
    },

    scrollNoteContentDown(): void {
      state.noteContentElement?.scrollBy({
        top: 50,
        behavior: 'smooth'
      });
    },

    scrollToSelectedInList,
    scrollToSelected
  };
}
