/**
 * Notification Utility
 * Simple utility for triggering global notifications.
 * Import and call notification.trigger() anywhere in the app.
 */

let notificationFunction:
  | ((message?: string, type?: 'error' | 'success' | 'info') => Promise<void>)
  | null = null

function register(
  fn: (message?: string, type?: 'error' | 'success' | 'info') => Promise<void>
): void {
  notificationFunction = fn
}

async function trigger(
  message?: string,
  type: 'error' | 'success' | 'info' = 'error'
): Promise<void> {
  if (notificationFunction) {
    try {
      await notificationFunction(message, type)
    } catch (e) {
      console.warn('Notification failed:', e)
    }
  } else {
    console.warn('notification: No notification function registered!')
  }
}

async function error(message?: string): Promise<void> {
  await trigger(message, 'error')
}

async function success(message?: string): Promise<void> {
  await trigger(message, 'success')
}

async function info(message?: string): Promise<void> {
  await trigger(message, 'info')
}

export const notification = {
  register,
  trigger,
  error,
  success,
  info,
}
