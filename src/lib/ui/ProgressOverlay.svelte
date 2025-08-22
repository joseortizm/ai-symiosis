<!--
UI Layer - Progress Overlay
Full-screen loading overlay with spinner and progress messages.
Dims the entire UI during database operations to prevent interaction.
-->

<script lang="ts">
  interface Props {
    show: boolean
    message: string
    error: string | null
  }

  const { show, message, error }: Props = $props()

  let overlayElement = $state<HTMLElement | undefined>(undefined)

  // Focus management for accessibility
  $effect(() => {
    if (show && overlayElement) {
      // Focus the overlay for screen readers
      setTimeout(() => overlayElement!.focus(), 10)
    }
  })
</script>

{#if show}
  <div
    class="progress-overlay"
    bind:this={overlayElement}
    tabindex="0"
    role="dialog"
    aria-modal="true"
    aria-live="polite"
    aria-label={error ? `Error: ${error}` : `Loading: ${message}`}
  >
    <div class="progress-content">
      {#if error}
        <!-- Error State -->
        <div class="progress-error">
          <div class="error-icon">⚠</div>
          <h3 class="error-title">Database Error</h3>
          <p class="error-message">{error}</p>
        </div>
      {:else}
        <!-- Loading State -->
        <div class="progress-loading">
          <div class="spinner"></div>
          <p class="progress-message">{message}</p>
        </div>
      {/if}
    </div>
  </div>
{/if}

<style>
  .progress-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background-color: rgba(0, 0, 0, 0.8);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1100;
    outline: none;
    animation: fadeIn 0.2s ease-out;
  }

  @keyframes fadeIn {
    from {
      opacity: 0;
    }
    to {
      opacity: 1;
    }
  }

  .progress-content {
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 32px;
    background-color: var(--theme-bg-primary);
    border: 1px solid var(--theme-bg-tertiary);
    border-radius: 12px;
    min-width: 320px;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
  }

  /* Loading State Styles */
  .progress-loading {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 24px;
  }

  .spinner {
    width: 48px;
    height: 48px;
    border: 4px solid var(--theme-bg-tertiary);
    border-top: 4px solid var(--theme-text-primary);
    border-radius: 50%;
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    from {
      transform: rotate(0deg);
    }
    to {
      transform: rotate(360deg);
    }
  }

  .progress-message {
    margin: 0;
    color: var(--theme-text-primary);
    font-size: 16px;
    text-align: center;
    line-height: 1.4;
    min-height: 22px; /* Prevent layout shift when message changes */
  }

  /* Error State Styles */
  .progress-error {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 16px;
    text-align: center;
  }

  .error-icon {
    font-size: 48px;
    color: #fb4934; /* Red accent for errors */
  }

  .error-title {
    margin: 0;
    color: #fb4934;
    font-size: 18px;
    font-weight: 600;
  }

  .error-message {
    margin: 0;
    color: var(--theme-text-primary);
    font-size: 14px;
    line-height: 1.5;
    max-width: 400px;
  }

  /* Responsive adjustments */
  @media (max-width: 480px) {
    .progress-content {
      margin: 16px;
      min-width: auto;
      max-width: calc(100vw - 32px);
    }
  }
</style>
