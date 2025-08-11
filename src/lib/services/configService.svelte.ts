/**
 * Service Layer - Config Service
 * Application configuration settings and the settings pane state.
 * Handles configuration loading, saving, and reactive settings panel visibility.
 */

import { invoke } from "@tauri-apps/api/core";

class ConfigService {
  state = $state({
    content: '',
    isVisible: false,
    isLoading: false,
    error: null as string | null
  });

  async open(): Promise<void> {
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

  close(): void {
    this.state.isVisible = false;
    this.state.content = '';
    this.state.error = null;
  }

  async save(): Promise<{ success: boolean; error?: string }> {
    this.state.isLoading = true;
    this.state.error = null;

    try {
      await invoke<void>("save_config_content", { content: this.state.content });
      await invoke<void>("refresh_cache");

      this.close();

      return { success: true };
    } catch (e) {
      const error = `Failed to save config: ${e}`;
      this.state.error = error;
      console.error("Failed to save config:", e);
      return { success: false, error };
    } finally {
      this.state.isLoading = false;
    }
  }

  // Update config content (for two-way binding)
  updateContent(content: string): void {
    this.state.content = content;
  }

  async exists(): Promise<boolean> {
    try {
      return await invoke<boolean>("config_exists");
    } catch (e) {
      console.error("Failed to check config existence:", e);
      return false;
    }
  }

  async refreshCache(): Promise<void> {
    try {
      await invoke<void>("refresh_cache");
    } catch (e) {
      console.error("Failed to refresh cache:", e);
      throw e;
    }
  }

  async getMarkdownTheme(): Promise<string> {
    try {
      return await invoke<string>("get_markdown_theme");
    } catch (e) {
      console.error("Failed to get markdown theme:", e);
      return "light";
    }
  }

  clearError(): void {
    this.state.error = null;
  }

  // Pane management methods for direct use in +page.svelte
  async openPane(): Promise<void> {
    await this.open();
  }

  closePane(): void {
    this.close();
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

export const configService = new ConfigService();
