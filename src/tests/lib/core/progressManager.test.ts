/**
 * Progress Manager Tests (Factory-based - TDD)
 * Tests for loading progress state management and UI coordination.
 */

import { beforeEach, describe, expect, it } from 'vitest'
import { createProgressManager } from '../../../lib/core/progressManager.svelte'

describe('progressManager (factory-based - TDD)', () => {
  let progressManager: ReturnType<typeof createProgressManager>

  beforeEach(() => {
    progressManager = createProgressManager()
  })

  describe('initial state', () => {
    it('should initialize with correct default state', () => {
      expect(progressManager.isLoading).toBe(false)
      expect(progressManager.message).toBe('')
      expect(progressManager.error).toBeNull()
      expect(progressManager.hasError).toBe(false)
      expect(progressManager.type).toBe('modal')
      expect(progressManager.showModal).toBe(false)
      expect(progressManager.showSubtle).toBe(false)
    })
  })

  describe('start', () => {
    it('should start loading with modal type by default', () => {
      progressManager.start('Loading data...')

      expect(progressManager.isLoading).toBe(true)
      expect(progressManager.message).toBe('Loading data...')
      expect(progressManager.error).toBeNull()
      expect(progressManager.type).toBe('modal')
      expect(progressManager.showModal).toBe(true)
      expect(progressManager.showSubtle).toBe(false)
    })

    it('should start loading with subtle type when specified', () => {
      progressManager.start('Loading data...', 'subtle')

      expect(progressManager.isLoading).toBe(true)
      expect(progressManager.message).toBe('Loading data...')
      expect(progressManager.type).toBe('subtle')
      expect(progressManager.showModal).toBe(false)
      expect(progressManager.showSubtle).toBe(true)
    })

    it('should clear any existing error when starting', () => {
      progressManager.setError('Previous error')
      expect(progressManager.error).toBe('Previous error')

      progressManager.start('Loading...')

      expect(progressManager.error).toBeNull()
      expect(progressManager.hasError).toBe(false)
    })
  })

  describe('updateProgress', () => {
    it('should update message when loading', () => {
      progressManager.start('Starting...')

      progressManager.updateProgress('Processing...')

      expect(progressManager.message).toBe('Processing...')
      expect(progressManager.isLoading).toBe(true)
    })

    it('should not update message when not loading', () => {
      progressManager.updateProgress('Should not update')

      expect(progressManager.message).toBe('')
      expect(progressManager.isLoading).toBe(false)
    })
  })

  describe('complete', () => {
    it('should reset all state when completed', () => {
      progressManager.start('Loading...', 'subtle')
      progressManager.updateProgress('Processing...')

      progressManager.complete()

      expect(progressManager.isLoading).toBe(false)
      expect(progressManager.message).toBe('')
      expect(progressManager.error).toBeNull()
      expect(progressManager.type).toBe('modal') // Reset to default
      expect(progressManager.showModal).toBe(false)
      expect(progressManager.showSubtle).toBe(false)
    })
  })

  describe('setError', () => {
    it('should set error and stop loading', () => {
      progressManager.start('Loading...')

      progressManager.setError('Something went wrong')

      expect(progressManager.isLoading).toBe(false)
      expect(progressManager.message).toBe('')
      expect(progressManager.error).toBe('Something went wrong')
      expect(progressManager.hasError).toBe(true)
      expect(progressManager.type).toBe('modal') // Errors always use modal
    })

    it('should force modal type for errors', () => {
      progressManager.start('Loading...', 'subtle')

      progressManager.setError('Error occurred')

      expect(progressManager.type).toBe('modal')
    })
  })

  describe('clearError', () => {
    it('should clear error state', () => {
      progressManager.setError('Test error')
      expect(progressManager.error).toBe('Test error')
      expect(progressManager.hasError).toBe(true)

      progressManager.clearError()

      expect(progressManager.error).toBeNull()
      expect(progressManager.hasError).toBe(false)
    })
  })

  describe('computed properties', () => {
    it('should show modal when loading with modal type', () => {
      progressManager.start('Loading...', 'modal')
      expect(progressManager.showModal).toBe(true)
      expect(progressManager.showSubtle).toBe(false)
    })

    it('should show subtle when loading with subtle type', () => {
      progressManager.start('Loading...', 'subtle')
      expect(progressManager.showModal).toBe(false)
      expect(progressManager.showSubtle).toBe(true)
    })

    it('should not show indicators when not loading', () => {
      progressManager.start('Loading...')
      progressManager.complete()

      expect(progressManager.showModal).toBe(false)
      expect(progressManager.showSubtle).toBe(false)
    })

    it('should compute hasError correctly', () => {
      expect(progressManager.hasError).toBe(false)

      progressManager.setError('Test error')
      expect(progressManager.hasError).toBe(true)

      progressManager.clearError()
      expect(progressManager.hasError).toBe(false)
    })
  })

  describe('workflow scenarios', () => {
    it('should handle complete loading workflow', () => {
      // Start loading
      progressManager.start('Initializing...', 'subtle')
      expect(progressManager.isLoading).toBe(true)
      expect(progressManager.showSubtle).toBe(true)

      // Update progress
      progressManager.updateProgress('Processing data...')
      expect(progressManager.message).toBe('Processing data...')

      // Complete successfully
      progressManager.complete()
      expect(progressManager.isLoading).toBe(false)
      expect(progressManager.showSubtle).toBe(false)
    })

    it('should handle error during loading', () => {
      progressManager.start('Loading data...')
      progressManager.updateProgress('Connecting to server...')

      progressManager.setError('Connection failed')

      expect(progressManager.isLoading).toBe(false)
      expect(progressManager.error).toBe('Connection failed')
      expect(progressManager.type).toBe('modal')
    })
  })

  describe('edge cases', () => {
    it('complete() when not loading should leave state unchanged', () => {
      // Initial state - not loading
      expect(progressManager.isLoading).toBe(false)
      expect(progressManager.message).toBe('')
      expect(progressManager.type).toBe('modal')

      progressManager.complete()

      // State should remain unchanged
      expect(progressManager.isLoading).toBe(false)
      expect(progressManager.message).toBe('')
      expect(progressManager.type).toBe('modal')
    })

    it('setError() immediately after complete() should still force modal state correctly', () => {
      progressManager.start('Loading...', 'subtle')
      progressManager.complete()

      progressManager.setError('Error after completion')

      expect(progressManager.isLoading).toBe(false)
      expect(progressManager.error).toBe('Error after completion')
      expect(progressManager.type).toBe('modal') // Should force modal for error
    })

    it('setError() called twice should overwrite the previous error', () => {
      progressManager.setError('First error')
      expect(progressManager.error).toBe('First error')

      progressManager.setError('Second error')
      expect(progressManager.error).toBe('Second error')
    })
  })
})
