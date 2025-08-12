<!--
UI Layer - Settings Pane
Configuration panel for editing application settings in TOML format.
Uses CodeMirror editor for syntax highlighting of configuration files.
-->

<script lang="ts">
  import CodeMirrorEditor from './CodeMirrorEditor.svelte';
  import { configService } from '../services/configService.svelte';
  import { getContext } from 'svelte';
  import type { AppManagers } from '../app/appCoordinator.svelte';

  interface Props {
    show: boolean;
    onClose: () => void;
    onRefresh: (notes: string[]) => void;
  }

  const { show, onClose, onRefresh }: Props = $props();
  const { searchManager } = getContext<AppManagers>('managers');

  let dialogElement = $state<HTMLElement | undefined>(undefined);

  async function handleSave(): Promise<void> {
    const result = await configService.save();

    if (result.success) {
      // Refresh the notes list after config change
      const notes = await searchManager.searchImmediate('');
      onRefresh(notes);
    }
  }

  function handleCancel(): void {
    configService.close();
    onClose();
  }

  function handleKeydown(event: KeyboardEvent): void {
    if (event.key === 'Escape') {
      event.preventDefault();
      handleCancel();
    } else if (event.ctrlKey && event.key === 's') {
      event.preventDefault();
      handleSave();
    }
  }

  function handleOverlayClick(e: MouseEvent): void {
    if (e.target === e.currentTarget) {
      handleCancel();
    }
  }

  $effect(() => {
    if (show && dialogElement) {
      setTimeout(() => dialogElement!.focus(), 10);
    }
  });
</script>

{#if show}
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="dialog-overlay" onclick={handleOverlayClick}>
    <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
    <div
      class="dialog settings-pane"
      bind:this={dialogElement}
      tabindex="0"
      onkeydown={handleKeydown}
      onclick={(e) => e.stopPropagation()}
    >
      <h3>Settings</h3>
      <div class="settings-editor-container">
        <CodeMirrorEditor
          bind:value={configService.content}
          filename="config.toml"
          onSave={handleSave}
          onContentChange={(newValue) => configService.content = newValue}
        />
      </div>
      <div class="keyboard-hint">
        <p>Press <kbd>Ctrl+S</kbd> to save, <kbd>Esc</kbd> in normal mode to close</p>
      </div>
      <div class="settings-buttons">
        <button class="btn-primary" onclick={handleSave}>Save</button>
        <button class="btn-cancel" onclick={handleCancel}>Cancel</button>
      </div>
    </div>
  </div>
{/if}

<style>
.dialog-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-color: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.dialog {
  background-color: #3c3836;
  border: 1px solid #504945;
  border-radius: 8px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.5);
  max-height: 90vh;
  overflow: auto;
  padding: 24px;
}

.dialog h3 {
  margin: 0 0 16px 0;
  color: #ebdbb2;
  font-size: 1.2em;
  font-weight: 600;
}

.settings-pane {
  width: 900px;
  max-width: 90vw;
  height: 500px;
  max-height: 80vh;
  display: flex;
  flex-direction: column;
}

.settings-editor-container {
  width: 100%;
  height: 500px;
  margin: 16px 0;
  border: 1px solid #504945;
  border-radius: 6px;
  overflow: hidden;
  background-color: #282828;
  flex: 1;
  min-height: 0;
}

.settings-buttons {
  display: flex;
  gap: 8px;
  justify-content: flex-end;
  margin-top: 16px;
}

.keyboard-hint {
  margin: 16px 0;
  padding: 12px;
  background-color: #32302f;
  border-radius: 4px;
  border-left: 3px solid #83a598;
}

.keyboard-hint p {
  margin: 4px 0;
  font-size: 13px;
  color: #a89984;
}

kbd {
  background-color: #504945;
  color: #ebdbb2;
  padding: 2px 6px;
  border-radius: 3px;
  font-size: 12px;
  font-family: 'JetBrains Mono', 'Fira Code', monospace;
  border: 1px solid #665c54;
  box-shadow: 0 1px 2px rgba(0, 0, 0, 0.2);
}

.settings-buttons button {
  padding: 8px 16px;
  border: none;
  border-radius: 4px;
  font-size: 14px;
  cursor: pointer;
  font-weight: 500;
}

.btn-cancel {
  background-color: #504945;
  color: #ebdbb2;
}

.btn-cancel:hover {
  background-color: #665c54;
}

.btn-primary {
  background-color: #b8bb26;
  color: #282828;
}

.btn-primary:hover:not(:disabled) {
  background-color: #98971a;
}

.btn-primary:disabled {
  background-color: #504945;
  color: #7c6f64;
  cursor: not-allowed;
}
</style>
