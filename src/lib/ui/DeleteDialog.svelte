<!--
UI Layer - Delete Dialog
Specialized confirmation dialog for note deletion with safety timeout mechanism.
Requires multiple key presses to confirm destructive actions.
-->

<script lang="ts">
  interface Props {
    show: boolean;
    noteName: string;
    deleteKeyPressCount: number;
    onConfirm?: () => void;
    onCancel?: () => void;
    onKeyPress?: () => void;
  }

  const { show, noteName, deleteKeyPressCount, onConfirm, onCancel, onKeyPress }: Props = $props();

  let dialogElement = $state<HTMLElement | undefined>(undefined);

  function handleConfirm(): void {
    onConfirm?.();
  }

  function handleCancel(): void {
    onCancel?.();
  }

  function handleKeyPress(): void {
    onKeyPress?.();
  }

  function handleKeydown(event: KeyboardEvent): void {
    if (event.key === 'Escape') {
      event.preventDefault();
      handleCancel();
    } else if (event.key === 'D' || event.key === 'd') {
      event.preventDefault();
      handleKeyPress();
    }
  }

  function handleOverlayClick(event: MouseEvent): void {
    if (event.target === event.currentTarget) {
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
      class="dialog"
      bind:this={dialogElement}
      tabindex="0"
      onkeydown={handleKeydown}
      onclick={(e) => e.stopPropagation()}
    >
      <h3>Delete Note</h3>
      <p>Are you sure you want to delete "{noteName}"?</p>
      <p class="warning">This action cannot be undone.</p>
      <div class="keyboard-hint">
        <p>Press <kbd>DD</kbd> to confirm or <kbd>Esc</kbd> to cancel</p>
        {#if deleteKeyPressCount === 1}
          <p class="delete-progress">Press <kbd>D</kbd> again to confirm deletion</p>
        {/if}
      </div>
      <div class="dialog-buttons">
        <button class="btn-cancel" onclick={handleCancel}>Cancel</button>
        <button class="btn-delete" onclick={handleConfirm}>Delete</button>
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
    background-color: rgba(0, 0, 0, 0.7);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }

  .dialog {
    background-color: #32302f;
    border: 1px solid #504945;
    border-radius: 6px;
    padding: 24px;
    min-width: 400px;
    max-width: 500px;
    color: #fbf1c7;
    outline: none;
  }

  .dialog h3 {
    margin: 0 0 16px 0;
    color: #fbf1c7;
    font-size: 18px;
    font-weight: 600;
  }

  .dialog p {
    margin: 0 0 12px 0;
    color: #d5c4a1;
    line-height: 1.5;
  }

  .warning {
    color: #fb4934 !important;
    font-size: 14px;
    font-style: italic;
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
    color: #ebdbb2;
    font-size: 14px;
  }

  .delete-progress {
    color: #fe8019 !important;
    font-weight: 500;
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

  .dialog-buttons {
    display: flex;
    gap: 12px;
    justify-content: flex-end;
    margin-top: 24px;
  }

  .dialog-buttons button {
    padding: 8px 16px;
    border-radius: 4px;
    border: none;
    font-size: 14px;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .btn-cancel {
    background-color: #504945;
    color: #ebdbb2;
    border: 1px solid #665c54;
  }

  .btn-cancel:hover {
    background-color: #665c54;
    border-color: #7c6f64;
  }

  .btn-cancel:focus {
    outline: 2px solid #83a598;
    outline-offset: 2px;
  }

  .btn-delete {
    background-color: #cc241d;
    color: #fbf1c7;
    border: 1px solid #fb4934;
  }

  .btn-delete:hover {
    background-color: #fb4934;
    border-color: #fe8019;
  }

  .btn-delete:focus {
    outline: 2px solid #83a598;
    outline-offset: 2px;
  }
</style>
