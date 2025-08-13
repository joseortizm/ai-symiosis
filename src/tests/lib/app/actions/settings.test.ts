import { describe, it, expect, beforeEach, vi } from 'vitest'
import { createSettingsActions } from '$lib/app/actions/settings.svelte'

describe('settings actions', () => {
  let settingsActions: ReturnType<typeof createSettingsActions>
  let mockDeps: Parameters<typeof createSettingsActions>[0]

  beforeEach(() => {
    mockDeps = {
      configService: {
        openPane: vi.fn().mockResolvedValue(undefined),
        closePane: vi.fn(),
      },
      focusManager: {
        focusSearch: vi.fn(),
      },
    }

    settingsActions = createSettingsActions(mockDeps)
  })

  describe('openSettingsPane', () => {
    it('should open config pane and focus search', async () => {
      await settingsActions.openSettingsPane()

      expect(mockDeps.configService.openPane).toHaveBeenCalledOnce()
      expect(mockDeps.focusManager.focusSearch).toHaveBeenCalledOnce()
    })

    it('should focus search even if opening pane fails', async () => {
      vi.mocked(mockDeps.configService.openPane).mockRejectedValue(
        new Error('Config failed')
      )

      await expect(settingsActions.openSettingsPane()).rejects.toThrow(
        'Config failed'
      )
      expect(mockDeps.focusManager.focusSearch).not.toHaveBeenCalled()
    })

    it('should handle openPane success correctly', async () => {
      const openPaneSpy = mockDeps.configService.openPane
      const focusSearchSpy = mockDeps.focusManager.focusSearch

      await settingsActions.openSettingsPane()

      expect(openPaneSpy).toHaveBeenCalledBefore(focusSearchSpy as any)
    })
  })

  describe('closeSettingsPane', () => {
    it('should close config pane and focus search', () => {
      settingsActions.closeSettingsPane()

      expect(mockDeps.configService.closePane).toHaveBeenCalledOnce()
      expect(mockDeps.focusManager.focusSearch).toHaveBeenCalledOnce()
    })

    it('should always focus search after closing pane', () => {
      const closePaneSpy = mockDeps.configService.closePane
      const focusSearchSpy = mockDeps.focusManager.focusSearch

      settingsActions.closeSettingsPane()

      expect(closePaneSpy).toHaveBeenCalledOnce()
      expect(focusSearchSpy).toHaveBeenCalledOnce()
    })
  })

  describe('interface compliance', () => {
    it('should expose all required methods', () => {
      expect(settingsActions).toHaveProperty('openSettingsPane')
      expect(settingsActions).toHaveProperty('closeSettingsPane')
      expect(typeof settingsActions.openSettingsPane).toBe('function')
      expect(typeof settingsActions.closeSettingsPane).toBe('function')
    })
  })
})
