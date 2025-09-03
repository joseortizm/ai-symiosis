/**
 * Error Notification Utility
 * Simple utility for triggering global error notifications.
 * Import and call errorNotification.trigger() anywhere in the app.
 */

let notificationFunction: ((message?: string) => Promise<void>) | null = null

function register(fn: (message?: string) => Promise<void>): void {
  notificationFunction = fn
}

async function trigger(message?: string): Promise<void> {
  if (notificationFunction) {
    try {
      await notificationFunction(message)
    } catch (e) {
      console.warn('Error notification failed:', e)
    }
  } else {
    console.warn('errorNotification: No notification function registered!')
  }
}

export const errorNotification = {
  register,
  trigger,
}
