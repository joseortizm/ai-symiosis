<!--
UI Layer - Editor Mode Manager
Handles loading and managing editor key binding modes (vim, emacs, basic).
-->

<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from "@tauri-apps/api/core";

  interface Props {
    onModeLoaded: (mode: string) => void;
  }

  let { onModeLoaded }: Props = $props();

  async function loadEditorMode(): Promise<void> {
    try {
      const mode = await invoke<string>("get_editor_mode");
      onModeLoaded(mode);
    } catch (e) {
      console.error("Failed to load editor mode:", e);
      onModeLoaded('basic');
    }
  }

  onMount(() => {
    loadEditorMode();
  });
</script>