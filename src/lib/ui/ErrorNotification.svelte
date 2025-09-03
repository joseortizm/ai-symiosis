<!--
UI Layer - Error Notifications
Elegant error toast notifications that slide in from the bottom right.
Used to provide visual feedback when operations fail silently.
-->

<script lang="ts">
  import { getCurrentWindow } from '@tauri-apps/api/window'
  import { errorNotification } from '../utils/errorNotification'

  let message = $state('')
  let showMessage = $state(false)
  let fadeOut = $state(false)
  let fadeTimer: ReturnType<typeof setTimeout> | null = null
  let hideTimer: ReturnType<typeof setTimeout> | null = null

  async function showError(errorMessage?: string): Promise<void> {
    // Clear any existing timers
    if (fadeTimer) clearTimeout(fadeTimer)
    if (hideTimer) clearTimeout(hideTimer)

    if (errorMessage) {
      message = errorMessage
      showMessage = true
      fadeOut = false
    }

    try {
      await getCurrentWindow().setFocus()
    } catch (e) {
      console.warn('OS-level attention failed', e)
    }

    // Start fade out animation after 6s, then hide after fade completes
    if (errorMessage) {
      fadeTimer = setTimeout(() => {
        fadeOut = true
        fadeTimer = null
      }, 6000)

      hideTimer = setTimeout(() => {
        showMessage = false
        fadeOut = false
        hideTimer = null
      }, 6500)
    }
  }

  errorNotification.register(showError)
</script>

{#if showMessage}
  <div class="error-toast" class:fade-out={fadeOut}>
    <div class="error-badge">
      <div class="error-icon">×</div>
      <div class="error-text">{message}</div>
    </div>
  </div>
{/if}

<style>
  .error-toast {
    position: fixed;
    bottom: 20px;
    right: 20px;
    z-index: 10000;
    pointer-events: none;
    animation: slideInFromRight 0.3s ease-out;
  }

  .error-badge {
    display: flex;
    align-items: center;
    gap: 8px;
    background: var(--theme-bg-secondary);
    border: 1px solid var(--theme-border);
    border-radius: 20px;
    padding: 8px 16px;
    font-family: var(--theme-font-family);
    font-size: 14px;
    color: var(--theme-text-primary);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
    max-width: 300px;
  }

  .error-icon {
    background: var(--theme-accent-error, #ff4444);
    color: white;
    width: 18px;
    height: 18px;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 12px;
    font-weight: bold;
    flex-shrink: 0;
  }

  .error-text {
    color: var(--theme-text-secondary);
    font-size: 13px;
    font-family: var(--editor-font-family);
  }

  @keyframes slideInFromRight {
    0% {
      opacity: 0;
      transform: translateX(100%);
    }
    100% {
      opacity: 1;
      transform: translateX(0);
    }
  }

  .error-toast.fade-out {
    animation: fadeOut 0.5s ease-out forwards;
  }

  @keyframes fadeOut {
    0% {
      opacity: 1;
      transform: translateX(0);
    }
    100% {
      opacity: 0;
      transform: translateX(20px);
    }
  }
</style>
