/**
 * CodeMirror Theme Management
 * Simple static imports and mapping for all available themes.
 */

import type { Extension } from '@codemirror/state'

// Import all themes from the bundle
import {
  abcdef,
  abyss,
  androidStudio,
  andromeda,
  basicDark,
  basicLight,
  forest,
  githubDark,
  githubLight,
  gruvboxDark,
  gruvboxLight,
  materialDark,
  materialLight,
  monokai,
  nord,
  palenight,
  solarizedDark,
  solarizedLight,
  tokyoNightDay,
  tokyoNightStorm,
  volcano,
  vsCodeDark,
  vsCodeLight,
} from '@fsegurai/codemirror-theme-bundle'

// Theme mapping
const themes: Record<string, Extension> = {
  abcdef: abcdef,
  abyss: abyss,
  'android-studio': androidStudio,
  andromeda: andromeda,
  'basic-dark': basicDark,
  'basic-light': basicLight,
  forest: forest,
  'github-dark': githubDark,
  'github-light': githubLight,
  'gruvbox-dark': gruvboxDark,
  'gruvbox-light': gruvboxLight,
  'material-dark': materialDark,
  'material-light': materialLight,
  monokai: monokai,
  nord: nord,
  palenight: palenight,
  'solarized-dark': solarizedDark,
  'solarized-light': solarizedLight,
  'tokyo-night-day': tokyoNightDay,
  'tokyo-night-storm': tokyoNightStorm,
  volcano: volcano,
  'vscode-dark': vsCodeDark,
  'vscode-light': vsCodeLight,
}

export type ThemeName = keyof typeof themes

/**
 * Gets a CodeMirror theme extension by name.
 * Falls back to gruvbox-dark if theme not found.
 */
export function getTheme(themeName: string): Extension {
  return themes[themeName] || themes['gruvbox-dark']
}

/**
 * Gets the list of all available theme names.
 */
export function getAvailableThemes(): string[] {
  return Object.keys(themes)
}

/**
 * Validates if a theme name is supported.
 */
export function isValidTheme(themeName: string): themeName is ThemeName {
  return themeName in themes
}
