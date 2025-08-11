/**
 * Service Layer - Note Service
 * Reactive CRUD operations for notes with loading state management.
 * Handles API calls to Rust backend for note creation, deletion, and renaming.
 */

import { invoke } from "@tauri-apps/api/core";

class NoteService {
  state = $state({
    isLoading: false,
    error: null as string | null,
    lastOperation: null as 'create' | 'delete' | 'rename' | null
  });

  async create(noteName: string): Promise<{ success: boolean; noteName?: string; error?: string }> {
    if (!noteName.trim()) return { success: false, error: 'Note name cannot be empty' };

    this.state.isLoading = true;
    this.state.error = null;
    this.state.lastOperation = 'create';

    try {
      // Auto-add .md extension if no extension provided
      const finalNoteName = noteName.includes('.') ? noteName : `${noteName}.md`;

      await invoke<void>("create_new_note", { noteName: finalNoteName });

      return { success: true, noteName: finalNoteName };
    } catch (e) {
      const error = `Failed to create note: ${e}`;
      this.state.error = error;
      console.error("Failed to create note:", e);
      return { success: false, error };
    } finally {
      this.state.isLoading = false;
    }
  }

  async delete(noteName: string): Promise<{ success: boolean; error?: string }> {
    if (!noteName) return { success: false, error: 'Note name cannot be empty' };

    this.state.isLoading = true;
    this.state.error = null;
    this.state.lastOperation = 'delete';

    try {
      await invoke<void>("delete_note", { noteName });

      return { success: true };
    } catch (e) {
      const error = `Failed to delete note: ${e}`;
      this.state.error = error;
      console.error("Failed to delete note:", e);
      return { success: false, error };
    } finally {
      this.state.isLoading = false;
    }
  }

  async rename(oldName: string, newName: string): Promise<{ success: boolean; newName?: string; error?: string }> {
    if (!newName.trim() || !oldName) return { success: false, error: 'Both old and new names are required' };

    this.state.isLoading = true;
    this.state.error = null;
    this.state.lastOperation = 'rename';

    try {
      // Auto-add .md extension if no extension provided
      const finalNewName = newName.includes('.') ? newName : `${newName}.md`;

      await invoke<void>("rename_note", { oldName, newName: finalNewName });

      return { success: true, newName: finalNewName };
    } catch (e) {
      const error = `Failed to rename note: ${e}`;
      this.state.error = error;
      console.error("Failed to rename note:", e);
      return { success: false, error };
    } finally {
      this.state.isLoading = false;
    }
  }

  async getContent(noteName: string): Promise<string> {
    try {
      return await invoke<string>("get_note_content", { noteName });
    } catch (e) {
      console.error("Failed to get note content:", e);
      throw e;
    }
  }

  async getRawContent(noteName: string): Promise<string> {
    try {
      return await invoke<string>("get_note_raw_content", { noteName });
    } catch (e) {
      console.error("Failed to get raw note content:", e);
      throw e;
    }
  }

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

  clearError(): void {
    this.state.error = null;
  }

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
