<!--
UI Layer - Input Dialog
Generic modal dialog for user text input with validation and keyboard support.
Used for note creation and renaming operations throughout the application.
-->

<script lang="ts">
  interface Props {
    show: boolean
    title?: string
    value: string
    placeholder?: string
    confirmText?: string
    cancelText?: string
    required?: boolean
    autoSelect?: boolean
    onConfirm?: (value: string) => void
    onCancel?: () => void
    onInput?: (value: string) => void
  }

  let {
    show,
    title = '',
    value = $bindable(),
    placeholder = '',
    confirmText = 'Confirm',
    cancelText = 'Cancel',
    required = true,
    autoSelect = false,
    onConfirm,
    onCancel,
    onInput,
  }: Props = $props()

  let inputElement = $state<HTMLInputElement | undefined>(undefined)
  let dialogElement = $state<HTMLElement | undefined>(undefined)

  function handleConfirm(): void {
    if (!confirmDisabled) {
      onConfirm?.(value)
    }
  }

  function handleCancel(): void {
    onCancel?.()
  }

  function handleInput(event: Event): void {
    const target = event.target as HTMLInputElement
    value = target.value
    onInput?.(value)
  }

  function handleKeydown(event: KeyboardEvent): void {
    if (event.key === 'Escape') {
      event.preventDefault()
      handleCancel()
    } else if (event.key === 'Enter') {
      event.preventDefault()
      event.stopPropagation()
      handleConfirm()
    }
  }

  function handleOverlayClick(event: MouseEvent): void {
    if (event.target === event.currentTarget) {
      handleCancel()
    }
  }

  let hasAutoSelected = $state(false)

  $effect(() => {
    if (show && inputElement) {
      setTimeout(() => {
        inputElement!.focus()
        if (autoSelect && !hasAutoSelected) {
          inputElement!.select()
          hasAutoSelected = true
        }
      }, 10)
    } else if (!show) {
      hasAutoSelected = false
    }
  })

  const confirmDisabled = $derived(required && !value.trim())
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
        <button class="cancel-btn" onclick={handleCancel}>
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

  .dialog-input {
    width: 100%;
    padding: 12px;
    border: 1px solid var(--theme-border);
    border-radius: 6px;
    background-color: var(--theme-bg-primary);
    color: var(--theme-text-primary);
    font-family: inherit;
    font-size: 14px;
    margin: 8px 0;
    box-sizing: border-box;
    outline: none;
    transition:
      border-color 0.2s ease,
      box-shadow 0.2s ease;
  }

  .dialog-input:focus {
    outline: none;
    border-color: var(--theme-accent);
    box-shadow: 0 0 0 2px rgba(131, 165, 152, 0.2);
  }

  .dialog-input::placeholder {
    color: var(--theme-text-muted);
  }

  .dialog-buttons {
    display: flex;
    gap: 6px;
    justify-content: flex-end;
    margin-top: 8px;
  }

  .cancel-btn,
  .confirm-btn {
    padding: 6px 12px;
    border: none;
    border-radius: 4px;
    font-size: 12px;
    cursor: pointer;
    font-weight: 500;
  }

  .cancel-btn {
    background-color: var(--theme-bg-tertiary);
    color: var(--theme-text-primary);
  }

  .cancel-btn:hover {
    background-color: var(--theme-border);
  }

  .confirm-btn {
    background-color: var(--theme-success);
    color: var(--theme-bg-primary);
  }

  .confirm-btn:hover:not(:disabled) {
    background-color: var(--theme-accent);
    filter: brightness(1.1);
  }

  .confirm-btn:disabled {
    background-color: var(--theme-bg-tertiary);
    color: var(--theme-text-muted);
    cursor: not-allowed;
  }
</style>
