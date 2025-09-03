<!--
UI Layer - Delete Dialog
Specialized confirmation dialog for note deletion with safety timeout mechanism.
Requires multiple key presses to confirm destructive actions.
-->

<script lang="ts">
  interface Props {
    show: boolean
    noteName: string
    deleteKeyPressCount: number
    onConfirm?: () => void
    onCancel?: () => void
    onKeyPress?: () => void
  }

  const {
    show,
    noteName,
    deleteKeyPressCount,
    onConfirm,
    onCancel,
    onKeyPress,
  }: Props = $props()

  let dialogElement = $state<HTMLElement | undefined>(undefined)

  function handleConfirm(): void {
    onConfirm?.()
  }

  function handleCancel(): void {
    onCancel?.()
  }

  function handleKeyPress(): void {
    onKeyPress?.()
  }

  function handleKeydown(event: KeyboardEvent): void {
    if (event.key === 'Escape') {
      event.preventDefault()
      handleCancel()
    } else if (event.key === 'D' || event.key === 'd') {
      event.preventDefault()
      handleKeyPress()
    }
  }

  function handleOverlayClick(event: MouseEvent): void {
    if (event.target === event.currentTarget) {
      handleCancel()
    }
  }

  $effect(() => {
    if (show && dialogElement) {
      setTimeout(() => dialogElement!.focus(), 10)
    }
  })
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
          <p class="delete-progress">
            Press <kbd>D</kbd> again to confirm deletion
          </p>
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
    background-color: var(--theme-bg-secondary);
    border: 1px solid var(--theme-border);
    border-radius: 8px;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.5);
    overflow: hidden;
    padding: 12px;
    min-width: 400px;
    max-width: 500px;
    color: var(--theme-text-primary);
    outline: none;
  }

  .dialog h3 {
    margin: 0 0 12px 0;
    color: var(--theme-text-primary);
    font-size: 1.3em;
    font-weight: 600;
  }

  .dialog p {
    margin: 0 0 12px 0;
    color: var(--theme-text-secondary);
    line-height: 1.5;
  }

  .warning {
    color: var(--theme-warning) !important;
    font-size: 14px;
    font-style: italic;
  }

  .keyboard-hint {
    margin: 6px 0;
    padding: 6px 8px;
    background-color: var(--theme-bg-primary);
    border-radius: 4px;
    border-left: 2px solid var(--theme-accent);
  }

  .keyboard-hint p {
    margin: 2px 0;
    font-size: 11px;
    color: var(--theme-text-secondary);
  }

  .delete-progress {
    color: var(--theme-highlight) !important;
    font-weight: 500;
  }

  kbd {
    background-color: var(--theme-bg-tertiary);
    color: var(--theme-text-primary);
    padding: 2px 6px;
    border-radius: 3px;
    font-size: 12px;
    font-family: 'JetBrains Mono', 'Fira Code', monospace;
    border: 1px solid var(--theme-border);
    box-shadow: 0 1px 2px rgba(0, 0, 0, 0.2);
    margin: 0 2px;
  }

  .dialog-buttons {
    display: flex;
    gap: 6px;
    justify-content: flex-end;
    margin-top: 8px;
  }

  .dialog-buttons button {
    padding: 6px 12px;
    border: none;
    border-radius: 4px;
    font-size: 12px;
    cursor: pointer;
    font-weight: 500;
  }

  .btn-cancel {
    background-color: var(--theme-bg-tertiary);
    color: var(--theme-text-primary);
  }

  .btn-cancel:hover {
    background-color: var(--theme-border);
  }

  .btn-delete {
    background-color: var(--theme-warning);
    color: var(--theme-bg-primary);
  }

  .btn-delete:hover {
    background-color: var(--theme-warning);
    filter: brightness(1.1);
  }
</style>
