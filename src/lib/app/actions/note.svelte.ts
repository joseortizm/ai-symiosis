import { tick } from "svelte";

interface NoteActionDeps {
  noteService: {
    create: (noteName: string) => Promise<{ success: boolean; noteName?: string; error?: string }>;
    delete: (noteName: string) => Promise<{ success: boolean; error?: string }>;
    rename: (oldName: string, newName: string) => Promise<{ success: boolean; newName?: string; error?: string }>;
  };
  searchManager: {
    searchInput: string;
    searchImmediate: (query: string) => Promise<string[]>;
    filteredNotes: string[];
    setFilteredNotes: (notes: string[]) => void;
  };
  dialogManager: {
    newNoteName: string;
    newNoteNameForRename: string;
    closeCreateDialog: () => void;
    closeDeleteDialog: () => void;
    closeRenameDialog: () => void;
  };
  focusManager: {
    focusSearch: () => void;
  };
  editorManager: {
    enterEditMode: (noteName: string, fallbackHtmlContent?: string, noteContentElement?: HTMLElement) => Promise<void>;
    saveNote: (noteName: string) => Promise<{ success: boolean; error?: string }>;
  };
  contentManager: {
    noteContent: string;
    refreshAfterSave: (noteName: string, query: string) => Promise<{ searchResults: string[] }>;
  };
  setSelectedIndex: (index: number) => void;
}

export function createNoteActions(deps: NoteActionDeps) {
  const {
    noteService,
    searchManager,
    dialogManager,
    focusManager,
    editorManager,
    contentManager,
    setSelectedIndex
  } = deps;

  async function createNote(noteNameParam?: string): Promise<void> {
    const inputNoteName = noteNameParam || dialogManager.newNoteName.trim();
    if (!inputNoteName.trim()) return;

    const result = await noteService.create(inputNoteName);

    if (result.success) {
      await searchManager.searchImmediate('');

      const noteIndex = searchManager.filteredNotes.findIndex(note => note === result.noteName);
      if (noteIndex >= 0) {
        setSelectedIndex(noteIndex);
      }

      dialogManager.closeCreateDialog();
      await tick();
      focusManager.focusSearch();

      await enterEditMode(result.noteName!);
    }
  }

  async function deleteNote(selectedNote: string | null): Promise<void> {
    if (!selectedNote) return;

    const result = await noteService.delete(selectedNote);

    if (result.success) {
      await searchManager.searchImmediate(searchManager.searchInput);
      dialogManager.closeDeleteDialog();
      await tick();
      focusManager.focusSearch();
    }
  }

  async function renameNote(selectedNote: string | null, newNameParam?: string): Promise<void> {
    const inputNewName = newNameParam || dialogManager.newNoteNameForRename.trim();
    if (!inputNewName.trim() || !selectedNote) return;

    const result = await noteService.rename(selectedNote, inputNewName);

    if (result.success) {
      await searchManager.searchImmediate(searchManager.searchInput);

      const noteIndex = searchManager.filteredNotes.findIndex(note => note === result.newName);
      if (noteIndex >= 0) {
        setSelectedIndex(noteIndex);
      }

      dialogManager.closeRenameDialog();
    }
  }

  async function enterEditMode(noteName: string): Promise<void> {
    await editorManager.enterEditMode(
      noteName,
      contentManager.noteContent,
      undefined
    );
  }

  async function saveNote(selectedNote: string | null): Promise<void> {
    if (!selectedNote) return;

    const result = await editorManager.saveNote(selectedNote);

    if (result.success) {
      try {
        const refreshResult = await contentManager.refreshAfterSave(selectedNote, searchManager.searchInput);
        searchManager.setFilteredNotes(refreshResult.searchResults);
      } catch (e) {
        console.error("Failed to refresh after save:", e);
      }
    } else {
      console.error("Failed to save note:", result.error);
    }
  }

  return {
    createNote,
    deleteNote,
    renameNote,
    enterEditMode,
    saveNote
  };
}
