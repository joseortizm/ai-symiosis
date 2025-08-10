<script lang="ts">
  interface Props {
    show: boolean;
    title?: string;
    value: string;
    placeholder?: string;
    confirmText?: string;
    cancelText?: string;
    required?: boolean;
    autoSelect?: boolean;
    onConfirm?: (value: string) => void;
    onCancel?: () => void;
    onInput?: (value: string) => void;
  }

  let {
    show,
    title = "",
    value = $bindable(),
    placeholder = "",
    confirmText = "Confirm",
    cancelText = "Cancel",
    required = true,
    autoSelect = false,
    onConfirm,
    onCancel,
    onInput
  }: Props = $props();

  let inputElement = $state<HTMLInputElement | undefined>(undefined);
  let dialogElement = $state<HTMLElement | undefined>(undefined);

  function handleConfirm(): void {
    if (!confirmDisabled) {
      onConfirm?.(value);
    }
  }

  function handleCancel(): void {
    onCancel?.();
  }

  function handleInput(event: Event): void {
    const target = event.target as HTMLInputElement;
    value = target.value;
    onInput?.(value);
  }

  function handleKeydown(event: KeyboardEvent): void {
    if (event.key === 'Escape') {
      event.preventDefault();
      handleCancel();
    } else if (event.key === 'Enter') {
      event.preventDefault();
      event.stopPropagation();
      handleConfirm();
    }
  }

  function handleOverlayClick(event: MouseEvent): void {
    if (event.target === event.currentTarget) {
      handleCancel();
    }
  }

  $effect(() => {
    if (show && inputElement) {
      setTimeout(() => {
        inputElement!.focus();
        if (autoSelect) {
          inputElement!.select();
        }
      }, 10);
    }
  });

  const confirmDisabled = $derived(required && !value.trim());
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
      <input
        bind:this={inputElement}
        bind:value
        {placeholder}
        class="dialog-input"
        oninput={handleInput}
        onkeydown={handleKeydown}
      />
      <div class="dialog-buttons">
        <button
          class="cancel-btn"
          onclick={handleCancel}
        >
          {cancelText} (Esc)
        </button>
        <button
          class="confirm-btn"
          onclick={handleConfirm}
          disabled={confirmDisabled}
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

  .dialog-input {
    width: 100%;
    padding: 12px;
    border: 1px solid #504945;
    border-radius: 6px;
    background-color: #282828;
    color: #ebdbb2;
    font-family: inherit;
    font-size: 14px;
    margin: 16px 0 24px 0;
    box-sizing: border-box;
    outline: none;
    transition: border-color 0.2s ease, box-shadow 0.2s ease;
  }

  .dialog-input:focus {
    outline: none;
    border-color: #83a598;
    box-shadow: 0 0 0 2px rgba(131, 165, 152, 0.2);
  }

  .dialog-input::placeholder {
    color: #928374;
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

  .confirm-btn:hover:not(:disabled) {
    background-color: #689d6a;
    border-color: #8ec07c;
  }

  .confirm-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .confirm-btn:focus {
    outline: 2px solid #83a598;
    outline-offset: 2px;
  }
</style>
