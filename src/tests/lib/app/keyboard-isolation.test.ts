/**
 * Tests for keyboard shortcut isolation when dialogs or settings are open
 */

import { describe, it, expect, vi, beforeEach } from 'vitest'
import { createKeyboardActions } from '../../../lib/app/actions/keyboard.svelte'
import type { AppState } from '../../../lib/app/actions/keyboard.svelte'

describe('Keyboard Shortcut Isolation', () => {
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  let mockDeps: any
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  let keyboardActions: any

  beforeEach(() => {
    mockDeps = {
      focusManager: {
        selectedIndex: 0,
        setSelectedIndex: vi.fn(),
      },
      loadNoteContent: vi.fn(),
      enterEditMode: vi.fn(),
      exitEditMode: vi.fn(),
      saveAndExitNote: vi.fn(),
      showExitEditDialog: vi.fn(),
      showDeleteDialog: vi.fn(),
      showCreateDialog: vi.fn(),
      showRenameDialog: vi.fn(),
      openSettingsPane: vi.fn(),
      clearHighlights: vi.fn(),
      clearSearch: vi.fn(),
      focusSearch: vi.fn(),
    }

    keyboardActions = createKeyboardActions(mockDeps)
  })

  describe('when settings pane is open', () => {
    it('should not process main UI shortcuts', async () => {
      const baseState: AppState = {
        isSearchInputFocused: false,
        isEditMode: false,
        isNoteContentFocused: false,
        filteredNotes: ['note1.md', 'note2.md'],
        selectedNote: 'note1.md',
        noteContentElement: document.createElement('div'),
        areHighlightsCleared: false,
        isEditorDirty: false,
        query: '',
        isSettingsOpen: true, // Settings is open
        isAnyDialogOpen: false,
      }

      const handler = keyboardActions.createKeyboardHandler(() => baseState)

      const event = new KeyboardEvent('keydown', { key: 'ArrowDown' })
      const preventDefaultSpy = vi.spyOn(event, 'preventDefault')

      await handler(event)

      // Should not process the arrow down shortcut
      expect(mockDeps.loadNoteContent).not.toHaveBeenCalled()
      expect(preventDefaultSpy).not.toHaveBeenCalled()
    })

    it('should not process cmd+, when settings already open', async () => {
      const baseState: AppState = {
        isSearchInputFocused: false,
        isEditMode: false,
        isNoteContentFocused: false,
        filteredNotes: ['note1.md'],
        selectedNote: 'note1.md',
        noteContentElement: document.createElement('div'),
        areHighlightsCleared: false,
        isEditorDirty: false,
        query: '',
        isSettingsOpen: true,
        isAnyDialogOpen: false,
      }

      const handler = keyboardActions.createKeyboardHandler(() => baseState)

      const event = new KeyboardEvent('keydown', { key: ',', metaKey: true })
      const preventDefaultSpy = vi.spyOn(event, 'preventDefault')

      await handler(event)

      // Should not call openSettingsPane again
      expect(mockDeps.openSettingsPane).not.toHaveBeenCalled()
      expect(preventDefaultSpy).not.toHaveBeenCalled()
    })
  })

  describe('when any dialog is open', () => {
    it('should not process main UI shortcuts when create dialog is open', async () => {
      const baseState: AppState = {
        isSearchInputFocused: false,
        isEditMode: false,
        isNoteContentFocused: false,
        filteredNotes: ['note1.md', 'note2.md'],
        selectedNote: 'note1.md',
        noteContentElement: document.createElement('div'),
        areHighlightsCleared: false,
        isEditorDirty: false,
        query: '',
        isSettingsOpen: false,
        isAnyDialogOpen: true, // Dialog is open
      }

      const handler = keyboardActions.createKeyboardHandler(() => baseState)

      const event = new KeyboardEvent('keydown', { key: 'ArrowDown' })
      const preventDefaultSpy = vi.spyOn(event, 'preventDefault')

      await handler(event)

      // Should not process the arrow down shortcut
      expect(mockDeps.loadNoteContent).not.toHaveBeenCalled()
      expect(preventDefaultSpy).not.toHaveBeenCalled()
    })

    it('should not process cmd+, when dialog is open', async () => {
      const baseState: AppState = {
        isSearchInputFocused: false,
        isEditMode: false,
        isNoteContentFocused: false,
        filteredNotes: ['note1.md'],
        selectedNote: 'note1.md',
        noteContentElement: document.createElement('div'),
        areHighlightsCleared: false,
        isEditorDirty: false,
        query: '',
        isSettingsOpen: false,
        isAnyDialogOpen: true,
      }

      const handler = keyboardActions.createKeyboardHandler(() => baseState)

      const event = new KeyboardEvent('keydown', { key: ',', metaKey: true })
      const preventDefaultSpy = vi.spyOn(event, 'preventDefault')

      await handler(event)

      // Should not call openSettingsPane
      expect(mockDeps.openSettingsPane).not.toHaveBeenCalled()
      expect(preventDefaultSpy).not.toHaveBeenCalled()
    })
  })

  describe('when no dialogs or settings are open', () => {
    it('should process shortcuts normally', async () => {
      const baseState: AppState = {
        isSearchInputFocused: false,
        isEditMode: false,
        isNoteContentFocused: false,
        filteredNotes: ['note1.md', 'note2.md'],
        selectedNote: 'note1.md',
        noteContentElement: document.createElement('div'),
        areHighlightsCleared: false,
        isEditorDirty: false,
        query: '',
        isSettingsOpen: false,
        isAnyDialogOpen: false,
      }

      const handler = keyboardActions.createKeyboardHandler(() => baseState)

      const event = new KeyboardEvent('keydown', { key: 'ArrowDown' })
      const preventDefaultSpy = vi.spyOn(event, 'preventDefault')

      await handler(event)

      // Should process the arrow down shortcut
      expect(mockDeps.loadNoteContent).toHaveBeenCalled()
      expect(preventDefaultSpy).toHaveBeenCalled()
    })

    it('should process cmd+, to open settings', async () => {
      const baseState: AppState = {
        isSearchInputFocused: false,
        isEditMode: false,
        isNoteContentFocused: false,
        filteredNotes: ['note1.md'],
        selectedNote: 'note1.md',
        noteContentElement: document.createElement('div'),
        areHighlightsCleared: false,
        isEditorDirty: false,
        query: '',
        isSettingsOpen: false,
        isAnyDialogOpen: false,
      }

      const handler = keyboardActions.createKeyboardHandler(() => baseState)

      const event = new KeyboardEvent('keydown', { key: ',', metaKey: true })
      const preventDefaultSpy = vi.spyOn(event, 'preventDefault')

      await handler(event)

      // Should call openSettingsPane
      expect(mockDeps.openSettingsPane).toHaveBeenCalled()
      expect(preventDefaultSpy).toHaveBeenCalled()
    })
  })
})
