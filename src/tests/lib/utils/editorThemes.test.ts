/**
 * Editor Themes Utility Tests
 * Tests for CodeMirror theme management and validation.
 */

import { describe, expect, it } from 'vitest'
import {
  getTheme,
  getAvailableThemes,
  isValidTheme,
  type ThemeName,
} from '../../../lib/utils/editorThemes'

describe('editorThemes utility', () => {
  describe('getTheme', () => {
    it('should return theme extension for valid theme name', () => {
      const theme = getTheme('gruvbox-dark')

      // Extension should be an object (CodeMirror extension)
      expect(typeof theme).toBe('object')
      expect(theme).not.toBeNull()
    })

    it('should return gruvbox-dark for valid built-in themes', () => {
      const validThemes = [
        'abcdef',
        'abyss',
        'android-studio',
        'andromeda',
        'basic-dark',
        'basic-light',
        'forest',
        'github-dark',
        'github-light',
        'gruvbox-light',
        'material-dark',
        'material-light',
        'monokai',
        'nord',
        'palenight',
        'solarized-dark',
        'solarized-light',
        'tokyo-night-day',
        'tokyo-night-storm',
        'volcano',
        'vscode-dark',
        'vscode-light',
      ]

      validThemes.forEach((themeName) => {
        const theme = getTheme(themeName)
        expect(typeof theme).toBe('object')
        expect(theme).not.toBeNull()
      })
    })

    it('should fallback to gruvbox-dark for invalid theme name', () => {
      const defaultTheme = getTheme('gruvbox-dark')
      const invalidTheme = getTheme('non-existent-theme')

      expect(invalidTheme).toEqual(defaultTheme)
    })

    it('should fallback to gruvbox-dark for empty string', () => {
      const defaultTheme = getTheme('gruvbox-dark')
      const emptyTheme = getTheme('')

      expect(emptyTheme).toEqual(defaultTheme)
    })

    it('should fallback to gruvbox-dark for undefined theme name', () => {
      const defaultTheme = getTheme('gruvbox-dark')
      const undefinedTheme = getTheme(undefined as unknown as string)

      expect(undefinedTheme).toEqual(defaultTheme)
    })

    it('should be case-sensitive', () => {
      const defaultTheme = getTheme('gruvbox-dark')
      const upperCaseTheme = getTheme('GRUVBOX-DARK')

      expect(upperCaseTheme).toEqual(defaultTheme) // Should fallback
    })
  })

  describe('getAvailableThemes', () => {
    it('should return array of theme names', () => {
      const themes = getAvailableThemes()

      expect(Array.isArray(themes)).toBe(true)
      expect(themes.length).toBeGreaterThan(0)
    })

    it('should include expected theme names', () => {
      const themes = getAvailableThemes()

      const expectedThemes = [
        'abcdef',
        'abyss',
        'android-studio',
        'andromeda',
        'basic-dark',
        'basic-light',
        'forest',
        'github-dark',
        'github-light',
        'gruvbox-dark',
        'gruvbox-light',
        'material-dark',
        'material-light',
        'monokai',
        'nord',
        'palenight',
        'solarized-dark',
        'solarized-light',
        'tokyo-night-day',
        'tokyo-night-storm',
        'volcano',
        'vscode-dark',
        'vscode-light',
      ]

      expectedThemes.forEach((themeName) => {
        expect(themes).toContain(themeName)
      })
    })

    it('should return same array on multiple calls', () => {
      const themes1 = getAvailableThemes()
      const themes2 = getAvailableThemes()

      expect(themes1).toEqual(themes2)
    })

    it('should return array with unique values', () => {
      const themes = getAvailableThemes()
      const uniqueThemes = [...new Set(themes)]

      expect(themes).toEqual(uniqueThemes)
    })
  })

  describe('isValidTheme', () => {
    it('should return true for valid theme names', () => {
      const validThemes = [
        'gruvbox-dark',
        'gruvbox-light',
        'github-dark',
        'github-light',
        'monokai',
        'nord',
        'vscode-dark',
        'vscode-light',
      ]

      validThemes.forEach((themeName) => {
        expect(isValidTheme(themeName)).toBe(true)
      })
    })

    it('should return false for invalid theme names', () => {
      const invalidThemes = [
        'invalid-theme',
        'non-existent',
        'GRUVBOX-DARK', // Case sensitive
        'gruvbox_dark', // Wrong separator
        '',
        ' ',
        'gruvbox-dark-extra',
      ]

      invalidThemes.forEach((themeName) => {
        expect(isValidTheme(themeName)).toBe(false)
      })
    })

    it('should return false for null and undefined', () => {
      expect(isValidTheme(null as unknown as string)).toBe(false)
      expect(isValidTheme(undefined as unknown as string)).toBe(false)
    })

    it('should return false for non-string values', () => {
      expect(isValidTheme(123 as unknown as string)).toBe(false)
      expect(isValidTheme(true as unknown as string)).toBe(false)
      expect(isValidTheme({} as unknown as string)).toBe(false)
      expect(isValidTheme([] as unknown as string)).toBe(false)
    })

    it('should have type guard functionality', () => {
      const unknownTheme: string = 'gruvbox-dark'

      if (isValidTheme(unknownTheme)) {
        // TypeScript should recognize this as ThemeName
        const typedTheme: ThemeName = unknownTheme
        expect(typedTheme).toBe('gruvbox-dark')
      }
    })
  })

  describe('theme consistency', () => {
    it('should have matching available themes and validation', () => {
      const availableThemes = getAvailableThemes()

      availableThemes.forEach((themeName) => {
        expect(isValidTheme(themeName)).toBe(true)
      })
    })

    it('should be able to get theme for all available themes', () => {
      const availableThemes = getAvailableThemes()

      availableThemes.forEach((themeName) => {
        const theme = getTheme(themeName)
        expect(typeof theme).toBe('object')
        expect(theme).not.toBeNull()
      })
    })

    it('should have default fallback theme in available themes', () => {
      const availableThemes = getAvailableThemes()
      expect(availableThemes).toContain('gruvbox-dark')
    })
  })

  describe('edge cases', () => {
    it('should handle themes with special characters in validation', () => {
      expect(isValidTheme('theme-with-dashes')).toBe(false) // Not in our theme list
      expect(isValidTheme('android-studio')).toBe(true) // Valid theme with dash
    })

    it('should handle whitespace in theme names', () => {
      expect(isValidTheme(' gruvbox-dark ')).toBe(false)
      expect(isValidTheme('gruvbox dark')).toBe(false)
      expect(isValidTheme('\tgruvbox-dark\n')).toBe(false)
    })

    it('should maintain immutability of available themes', () => {
      const themes1 = getAvailableThemes()
      const themes2 = getAvailableThemes()

      // Modify the first array
      themes1.push('fake-theme')

      // Second array should be unaffected
      expect(themes2).not.toContain('fake-theme')
    })

    it('should handle concurrent access to themes', () => {
      const promises = Array.from({ length: 10 }, () =>
        Promise.resolve(getAvailableThemes())
      )

      return Promise.all(promises).then((results) => {
        const firstResult = results[0]
        results.forEach((result) => {
          expect(result).toEqual(firstResult)
        })
      })
    })
  })

  describe('performance considerations', () => {
    it('should return available themes quickly', () => {
      const start = performance.now()
      const themes = getAvailableThemes()
      const end = performance.now()

      expect(themes.length).toBeGreaterThan(0)
      expect(end - start).toBeLessThan(10) // Should be very fast
    })

    it('should validate themes quickly', () => {
      const start = performance.now()

      // Test multiple validations
      for (let i = 0; i < 100; i++) {
        isValidTheme('gruvbox-dark')
        isValidTheme('invalid-theme')
      }

      const end = performance.now()
      expect(end - start).toBeLessThan(100) // Should handle many validations quickly
    })
  })
})
