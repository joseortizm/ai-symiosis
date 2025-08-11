import { describe, it, expect, beforeEach, vi } from 'vitest';

// Mock DOM elements
class MockHTMLElement {
  focus = vi.fn();
  scrollBy = vi.fn();
  scrollIntoView = vi.fn();
  children: any[] = [];
}

const { createFocusManager } = await import('../../../lib/core/focusManager.svelte');

// Create a fresh instance for each test
let focusManager: ReturnType<typeof createFocusManager>;

describe('focusManager', () => {
  let searchElement: MockHTMLElement;
  let noteContentElement: MockHTMLElement;
  let noteListElement: MockHTMLElement;

  beforeEach(() => {
    // Create fresh manager instance
    focusManager = createFocusManager();

    // Create fresh mock elements
    searchElement = new MockHTMLElement();
    noteContentElement = new MockHTMLElement();
    noteListElement = new MockHTMLElement();

    // Reset focus manager state
    focusManager.setSearchInputFocused(false);
    focusManager.setNoteContentFocused(false);
    focusManager.setSearchElement(null);
    focusManager.setNoteContentElement(null);
    focusManager.setNoteListElement(null);

    vi.clearAllMocks();
  });

  describe('state getters', () => {
    it('should initialize with default state', () => {
      expect(focusManager.isSearchInputFocused).toBe(false);
      expect(focusManager.isNoteContentFocused).toBe(false);
      expect(focusManager.searchElement).toBeNull();
      expect(focusManager.noteContentElement).toBeNull();
      expect(focusManager.noteListElement).toBeNull();
    });
  });

  describe('focus state management', () => {
    it('should update search input focus state', () => {
      focusManager.setSearchInputFocused(true);
      expect(focusManager.isSearchInputFocused).toBe(true);

      focusManager.setSearchInputFocused(false);
      expect(focusManager.isSearchInputFocused).toBe(false);
    });

    it('should update note content focus state', () => {
      focusManager.setNoteContentFocused(true);
      expect(focusManager.isNoteContentFocused).toBe(true);

      focusManager.setNoteContentFocused(false);
      expect(focusManager.isNoteContentFocused).toBe(false);
    });
  });

  describe('element management', () => {
    it('should set and get elements', () => {
      focusManager.setSearchElement(searchElement as any);
      focusManager.setNoteContentElement(noteContentElement as any);
      focusManager.setNoteListElement(noteListElement as any);

      expect(focusManager.searchElement).toBe(searchElement);
      expect(focusManager.noteContentElement).toBe(noteContentElement);
      expect(focusManager.noteListElement).toBe(noteListElement);
    });
  });

  describe('focus actions', () => {
    beforeEach(() => {
      focusManager.setSearchElement(searchElement as any);
      focusManager.setNoteContentElement(noteContentElement as any);
    });

    it('should focus search input', () => {
      focusManager.focusSearch();
      expect(searchElement.focus).toHaveBeenCalledOnce();
    });

    it('should handle null search element gracefully', () => {
      focusManager.setSearchElement(null);
      expect(() => focusManager.focusSearch()).not.toThrow();
    });
  });

  describe('scroll actions', () => {
    beforeEach(() => {
      focusManager.setNoteContentElement(noteContentElement as any);
      focusManager.setNoteListElement(noteListElement as any);
    });

    it('should scroll note content up', () => {
      focusManager.scrollNoteContentUp();
      expect(noteContentElement.scrollBy).toHaveBeenCalledWith({
        top: -50,
        behavior: 'smooth'
      });
    });

    it('should scroll note content down', () => {
      focusManager.scrollNoteContentDown();
      expect(noteContentElement.scrollBy).toHaveBeenCalledWith({
        top: 50,
        behavior: 'smooth'
      });
    });

    it('should scroll to selected item in list', () => {
      const mockButton = new MockHTMLElement();
      const mockLi = { querySelector: vi.fn().mockReturnValue(mockButton) };
      noteListElement.children = [mockLi];

      focusManager.scrollToSelectedInList(0);

      expect(mockLi.querySelector).toHaveBeenCalledWith('button');
      expect(mockButton.scrollIntoView).toHaveBeenCalledWith({
        behavior: 'smooth',
        block: 'nearest'
      });
    });

    it('should handle null elements for scroll operations', () => {
      focusManager.setNoteContentElement(null);
      focusManager.setNoteListElement(null);

      expect(() => focusManager.scrollNoteContentUp()).not.toThrow();
      expect(() => focusManager.scrollToSelectedInList(0)).not.toThrow();
    });

    it('should scroll to selected item by index', () => {
      const mockButton = new MockHTMLElement();
      const mockLi = { querySelector: vi.fn().mockReturnValue(mockButton) };
      noteListElement.children = [mockLi];

      focusManager.scrollToSelected(0);

      expect(mockLi.querySelector).toHaveBeenCalledWith('button');
      expect(mockButton.scrollIntoView).toHaveBeenCalledWith({
        behavior: 'smooth',
        block: 'nearest'
      });
    });
  });
});
