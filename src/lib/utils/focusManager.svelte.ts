interface FocusState {
  isSearchInputFocused: boolean;
  isNoteContentFocused: boolean;
  searchElement: HTMLInputElement | null;
  noteContentElement: HTMLElement | null;
  noteListElement: HTMLElement | null;
}

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

export const focusManager = {
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
