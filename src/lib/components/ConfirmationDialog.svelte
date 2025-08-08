<script lang="ts">
  import { createEventDispatcher } from 'svelte';

  export let show: boolean = false;
  export let title: string = "Confirm";
  export let message: string = "";
  export let confirmText: string = "Confirm";
  export let cancelText: string = "Cancel";
  export let variant: 'default' | 'danger' = "default";

  const dispatch = createEventDispatcher<{
    confirm: void;
    cancel: void;
  }>();

  let dialogElement: HTMLElement;

  function handleConfirm(): void {
    dispatch('confirm');
  }

  function handleCancel(): void {
    dispatch('cancel');
  }

  function handleKeydown(event: KeyboardEvent): void {
    if (event.key === 'Escape') {
      event.preventDefault();
      handleCancel();
    } else if (event.key === 'Enter') {
      event.preventDefault();
      handleConfirm();
    }
  }

  function handleOverlayClick(event: MouseEvent): void {
    if (event.target === event.currentTarget) {
      handleCancel();
    }
  }

  $: if (show && dialogElement) {
    setTimeout(() => dialogElement.focus(), 10);
  }
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
      <h3>{title}</h3>
      <p class="dialog-message">{message}</p>
      <div class="dialog-buttons">
        <button
          class="cancel-btn"
          onclick={handleCancel}
        >
          {cancelText} (Esc)
        </button>
        <button
          class="confirm-btn {variant === 'danger' ? 'danger' : ''}"
          onclick={handleConfirm}
        >
          {confirmText} (Enter)
        </button>
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

  .dialog-message {
    margin: 0 0 24px 0;
    color: #ebdbb2;
    line-height: 1.5;
  }

  .dialog-buttons {
    display: flex;
    gap: 12px;
    justify-content: center;
  }

  .cancel-btn, .confirm-btn {
    padding: 8px 16px;
    border-radius: 4px;
    cursor: pointer;
    font-family: inherit;
    font-size: 14px;
    min-width: 140px;
    text-align: center;
  }

  .cancel-btn {
    border: 1px solid #665c54;
    background-color: #3c3836;
    color: #fbf1c7;
  }

  .cancel-btn:hover {
    background-color: #504945;
    border-color: #7c6f64;
  }

  .cancel-btn:focus {
    outline: 2px solid #83a598;
    outline-offset: 2px;
  }

  .confirm-btn {
    border: 1px solid #83a598;
    background-color: #458588;
    color: #fbf1c7;
    font-weight: 500;
  }

  .confirm-btn:hover {
    background-color: #689d6a;
    border-color: #8ec07c;
  }

  .confirm-btn.danger {
    background-color: #cc241d;
    border-color: #fb4934;
  }

  .confirm-btn.danger:hover {
    background-color: #fb4934;
    border-color: #fe8019;
  }

  .confirm-btn:focus {
    outline: 2px solid #83a598;
    outline-offset: 2px;
  }

</style>
