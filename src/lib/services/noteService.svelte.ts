import { invoke } from "@tauri-apps/api/core";
import { tick } from "svelte";

class NoteService {
  // Reactive state using Svelte 5 runes
  state = $state({
    isLoading: false,
    error: null as string | null,
    lastOperation: null as 'create' | 'delete' | 'rename' | null
  });

  // Create a new note with automatic extension handling
  async create(noteName: string, searchManager: any, dialogManager: any, onRefresh: (notes: string[]) => void, onFocus?: () => void): Promise<void> {
    if (!noteName.trim()) return;

    this.state.isLoading = true;
    this.state.error = null;
    this.state.lastOperation = 'create';

    try {
      // Auto-add .md extension if no extension provided
      const finalNoteName = noteName.includes('.') ? noteName : `${noteName}.md`;
      
      await invoke<void>("create_new_note", { noteName: finalNoteName });
      
      // Refresh the notes list
      const notes = await searchManager.searchImmediate('');
      onRefresh(notes);
      
      dialogManager.closeCreateDialog();
      
      // Return focus to search
      await tick();
      onFocus?.();
    } catch (e) {
      this.state.error = `Failed to create note: ${e}`;
      console.error("Failed to create note:", e);
      throw e;
    } finally {
      this.state.isLoading = false;
    }
  }

  // Delete a note
  async delete(noteName: string, searchManager: any, dialogManager: any, onRefresh: (notes: string[]) => void, currentSearchInput: string, onFocus?: () => void): Promise<void> {
    if (!noteName) return;

    this.state.isLoading = true;
    this.state.error = null;
    this.state.lastOperation = 'delete';

    try {
      await invoke<void>("delete_note", { noteName });
      
      // Refresh the notes list
      const notes = await searchManager.searchImmediate(currentSearchInput);
      onRefresh(notes);
      
      dialogManager.closeDeleteDialog();
      
      // Return focus to search
      await tick();
      onFocus?.();
    } catch (e) {
      this.state.error = `Failed to delete note: ${e}`;
      console.error("Failed to delete note:", e);
      throw e;
    } finally {
      this.state.isLoading = false;
    }
  }

  // Rename a note
  async rename(oldName: string, newName: string, searchManager: any, dialogManager: any, onRefresh: (notes: string[]) => void, onSelectNote: (noteName: string) => void, currentSearchInput: string): Promise<void> {
    if (!newName.trim() || !oldName) return;

    this.state.isLoading = true;
    this.state.error = null;
    this.state.lastOperation = 'rename';

    try {
      // Auto-add .md extension if no extension provided
      const finalNewName = newName.includes('.') ? newName : `${newName}.md`;
      
      await invoke<void>("rename_note", { oldName, newName: finalNewName });
      
      // Refresh the notes list
      const notes = await searchManager.searchImmediate(currentSearchInput);
      onRefresh(notes);
      
      // Select the renamed note
      onSelectNote(finalNewName);
      
      dialogManager.closeRenameDialog();
    } catch (e) {
      this.state.error = `Failed to rename note: ${e}`;
      console.error("Failed to rename note:", e);
      throw e;
    } finally {
      this.state.isLoading = false;
    }
  }

  // Get note content for display
  async getContent(noteName: string): Promise<string> {
    try {
      return await invoke<string>("get_note_content", { noteName });
    } catch (e) {
      console.error("Failed to get note content:", e);
      throw e;
    }
  }

  // Get raw note content for editing
  async getRawContent(noteName: string): Promise<string> {
    try {
      return await invoke<string>("get_note_raw_content", { noteName });
    } catch (e) {
      console.error("Failed to get raw note content:", e);
      throw e;
    }
  }

  // Save note content
  async save(noteName: string, content: string): Promise<void> {
    try {
      await invoke<void>("save_note", { noteName, content });
    } catch (e) {
      console.error("Failed to save note:", e);
      throw e;
    }
  }

  // System integration - open in external editor
  async openInEditor(noteName: string): Promise<void> {
    try {
      await invoke("open_note_in_editor", { noteName });
    } catch (e) {
      console.error("Failed to open note in editor:", e);
      throw e;
    }
  }

  // System integration - open note folder
  async openFolder(noteName: string): Promise<void> {
    try {
      await invoke("open_note_folder", { noteName });
    } catch (e) {
      console.error("Failed to open note folder:", e);
      throw e;
    }
  }

  // Clear any error state
  clearError(): void {
    this.state.error = null;
  }

  // Getters for reactive state
  get isLoading(): boolean {
    return this.state.isLoading;
  }

  get error(): string | null {
    return this.state.error;
  }

  get lastOperation(): string | null {
    return this.state.lastOperation;
  }
}

// Export singleton instance
export const noteService = new NoteService();