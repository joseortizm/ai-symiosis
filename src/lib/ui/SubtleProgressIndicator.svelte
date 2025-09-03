<!--
UI Layer - Subtle Progress Indicator
Small, non-intrusive loading spinner positioned in bottom-right corner.
Used for quick operations like search where modal overlays would be disruptive.
-->

<script lang="ts">
  interface Props {
    show: boolean
    message?: string
  }

  const { show, message }: Props = $props()

  let visible = $state(false)
  let showTimeout: ReturnType<typeof setTimeout> | undefined
  let hideTimeout: ReturnType<typeof setTimeout> | undefined

  // Debounced visibility management
  $effect(() => {
    if (show) {
      // Clear any pending hide timeout
      if (hideTimeout) {
        clearTimeout(hideTimeout)
        hideTimeout = undefined
      }

      // Show after 150ms delay to prevent flicker on very quick operations
      showTimeout = setTimeout(() => {
        visible = true
      }, 150)
    } else {
      // Clear show timeout if operation completes before delay
      if (showTimeout) {
        clearTimeout(showTimeout)
        showTimeout = undefined
      }

      // If already visible, maintain minimum display time of 300ms
      if (visible) {
        hideTimeout = setTimeout(() => {
          visible = false
        }, 300)
      }
    }

    // Cleanup function
    return () => {
      if (showTimeout) clearTimeout(showTimeout)
      if (hideTimeout) clearTimeout(hideTimeout)
    }
  })
</script>

{#if visible}
  <div
    class="subtle-progress-indicator"
    role="status"
    aria-live="polite"
    aria-label="Loading"
  >
    <div class="spinner"></div>
    {#if message}
      <div class="message">{message}</div>
    {/if}
  </div>
{/if}

<style>
  .subtle-progress-indicator {
    position: fixed;
    bottom: 20px;
    right: 20px;
    display: flex;
    align-items: center;
    gap: 8px;
    background-color: rgba(0, 0, 0, 0.8);
    color: white;
    padding: 8px 12px;
    border-radius: 20px;
    z-index: 500;
    font-size: 12px;
    backdrop-filter: blur(4px);
    animation: fadeIn 0.2s ease-out;
    pointer-events: none; /* Non-intrusive */
  }

  @keyframes fadeIn {
    from {
      opacity: 0;
      transform: translateY(10px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  .spinner {
    width: 16px;
    height: 16px;
    border: 2px solid rgba(255, 255, 255, 0.3);
    border-top: 2px solid white;
    border-radius: 50%;
    animation: spin 1s linear infinite;
    flex-shrink: 0;
  }

  @keyframes spin {
    from {
      transform: rotate(0deg);
    }
    to {
      transform: rotate(360deg);
    }
  }

  .message {
    white-space: nowrap;
    max-width: 150px;
    overflow: hidden;
    text-overflow: ellipsis;
    opacity: 0.9;
  }

  /* Responsive adjustments */
  @media (max-width: 768px) {
    .subtle-progress-indicator {
      bottom: 16px;
      right: 16px;
      padding: 6px 10px;
      font-size: 11px;
    }

    .spinner {
      width: 14px;
      height: 14px;
    }

    .message {
      max-width: 120px;
    }
  }
</style>
