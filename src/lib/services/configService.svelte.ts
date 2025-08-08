import { invoke } from "@tauri-apps/api/core";

class ConfigService {
  // Reactive state using Svelte 5 runes
  state = $state({
    content: '',
    isVisible: false,
    isLoading: false,
    error: null as string | null
  });

  // Load config content and show settings pane
  async open(onFocus?: () => void): Promise<void> {
    this.state.isLoading = true;
    this.state.error = null;

    try {
      const content = await invoke<string>("get_config_content");
      this.state.content = content;
      this.state.isVisible = true;
    } catch (e) {
      this.state.error = `Failed to load config: ${e}`;
      console.error("Failed to load config:", e);
    } finally {
      this.state.isLoading = false;
    }
  }

  // Close settings pane and clear content
  close(onFocus?: () => void): void {
    this.state.isVisible = false;
    this.state.content = '';
    this.state.error = null;
    onFocus?.();
  }

  // Save config content and refresh cache
  async save(searchManager: any, onRefresh: (notes: string[]) => void, onFocus?: () => void): Promise<void> {
    this.state.isLoading = true;
    this.state.error = null;

    try {
      await invoke<void>("save_config_content", { content: this.state.content });
      await invoke<void>("refresh_cache");
      
      this.close(onFocus);
      
      // Refresh the notes list after config change
      const notes = await searchManager.searchImmediate('');
      onRefresh(notes);
    } catch (e) {
      this.state.error = `Failed to save config: ${e}`;
      console.error("Failed to save config:", e);
      throw e;
    } finally {
      this.state.isLoading = false;
    }
  }

  // Update config content (for two-way binding)
  updateContent(content: string): void {
    this.state.content = content;
  }

  // Check if config file exists
  async exists(): Promise<boolean> {
    try {
      return await invoke<boolean>("config_exists");
    } catch (e) {
      console.error("Failed to check config existence:", e);
      return false;
    }
  }

  // Refresh cache manually
  async refreshCache(): Promise<void> {
    try {
      await invoke<void>("refresh_cache");
    } catch (e) {
      console.error("Failed to refresh cache:", e);
      throw e;
    }
  }

  // Clear any error state
  clearError(): void {
    this.state.error = null;
  }

  // Getters and setters for reactive state (to support bind:value)
  get content(): string {
    return this.state.content;
  }

  set content(value: string) {
    this.state.content = value;
  }

  get isVisible(): boolean {
    return this.state.isVisible;
  }

  get isLoading(): boolean {
    return this.state.isLoading;
  }

  get error(): string | null {
    return this.state.error;
  }
}

// Export singleton instance
export const configService = new ConfigService();