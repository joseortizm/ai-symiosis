<!--
UI Layer - CodeMirror Editor Core
Focused component handling CodeMirror initialization and content editing.
-->

<script lang="ts">
  import { onMount, tick, getContext } from 'svelte'
  import type { AppManagers } from '../app/appCoordinator.svelte'
  import { EditorView, basicSetup } from 'codemirror'
  import type { Extension } from '@codemirror/state'
  import { keymap } from '@codemirror/view'
  import { indentWithTab } from '@codemirror/commands'
  import { indentUnit } from '@codemirror/language'
  import { EditorState } from '@codemirror/state'
  import {
    codeFolding,
    foldState,
    foldCode,
    unfoldCode,
    foldAll,
    unfoldAll,
    foldable,
    foldEffect,
    syntaxTree,
  } from '@codemirror/language'
  import { markdown, markdownLanguage } from '@codemirror/lang-markdown'
  import { languages } from '@codemirror/language-data'
  import { StreamLanguage } from '@codemirror/language'
  import { toml } from '@codemirror/legacy-modes/mode/toml'
  import { vim, Vim } from '@replit/codemirror-vim'
  import { emacs } from '@replit/codemirror-emacs'
  import { getTheme } from '../utils/editorThemes'
  import { EditorView as EditorViewBase } from '@codemirror/view'

  interface Props {
    value: string
    filename: string
    nearestHeaderText?: string
    onSave: () => void
    onContentChange?: (newValue: string) => void
    onDirtyChange?: (isDirty: boolean) => void
    onExit?: (() => void) | null | undefined
    onRequestExit?: (() => void) | null | undefined
    isDirty?: boolean
  }

  let {
    value = $bindable(),
    filename,
    nearestHeaderText = '',
    onSave,
    onContentChange,
    onExit = null,
    onRequestExit = null,
    isDirty = $bindable(false),
  }: Props = $props()

  // Get reactive config state
  const { configStateManager } = getContext<AppManagers>('managers')

  let editorContainer: HTMLElement
  let editorView: EditorView | null = null
  let initialValue = $state(value)
  let lastPropsValue = $state(value)

  // Reactive config values
  const keyBindingMode = $derived(configStateManager.editor.mode || 'basic')
  const currentTheme = $derived(
    getTheme(configStateManager.editor.theme || 'gruvbox-dark')
  )
  const editorFontFamily = $derived(
    configStateManager.interface.editor_font_family ||
      'JetBrains Mono, Consolas, monospace'
  )
  const editorFontSize = $derived(
    configStateManager.interface.editor_font_size || 14
  )

  const propsChanged = $derived(value !== lastPropsValue)

  function handleDirtyChange(dirty: boolean): void {
    isDirty = dirty
  }

  // Use effect only for side effect (notification), not state updates
  $effect(() => {
    if (propsChanged) {
      handleDirtyChange(false)
    }
  })

  function resetDirtyFlag(): void {
    initialValue = value
    lastPropsValue = value
    handleDirtyChange(false)
  }

  function createFontExtension(
    fontFamily: string,
    fontSize: number
  ): Extension {
    return EditorViewBase.theme({
      '&': {
        fontFamily: fontFamily,
        fontSize: `${fontSize}px`,
      },
      '.cm-content': {
        fontFamily: fontFamily,
        fontSize: `${fontSize}px`,
      },
      '.cm-editor': {
        fontFamily: fontFamily,
        fontSize: `${fontSize}px`,
      },
    })
  }

  function getKeyMappingsMode(mode: string): Extension | null {
    switch (mode) {
      case 'vim':
        return vim({
          status: false,
        })
      case 'emacs':
        return emacs()
      case 'basic':
        return null
      default:
        return null
    }
  }

  function setupVimFoldingCommands(): void {
    // Define vim folding actions
    Vim.defineAction('foldClose', (cm: unknown) => {
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      const view = (cm as any).cm6 || cm
      foldCode(view)
    })

    Vim.defineAction('foldOpen', (cm: unknown) => {
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      const view = (cm as any).cm6 || cm
      unfoldCode(view)
    })

    Vim.defineAction('foldToggle', (cm: unknown) => {
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      const view = (cm as any).cm6 || cm
      const state = view.state

      // Try to unfold first, if nothing happens then fold
      const beforeFolds = state.field(foldState, false)?.size || 0
      unfoldCode(view)
      const afterUnfold = view.state.field(foldState, false)?.size || 0

      // If no change occurred, then nothing was unfolded, so fold instead
      if (beforeFolds === afterUnfold) {
        foldCode(view)
      }
    })

    Vim.defineAction('foldCloseAll', (cm: unknown) => {
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      const view = (cm as any).cm6 || cm
      // Sensible fold like vim's zM - only fold headers (h2+), code blocks, and lists
      const state = view.state
      const foldRanges: { from: number; to: number }[] = []

      syntaxTree(state).iterate({
        enter(node) {
          // Only fold sensible markdown structures
          const shouldFold =
            // All headers (including H1)
            node.name === 'ATXHeading1' ||
            node.name === 'ATXHeading2' ||
            node.name === 'ATXHeading3' ||
            node.name === 'ATXHeading4' ||
            node.name === 'ATXHeading5' ||
            node.name === 'ATXHeading6' ||
            node.name === 'SetextHeading1' ||
            node.name === 'SetextHeading2' ||
            // Code blocks
            node.name === 'FencedCode' ||
            node.name === 'CodeBlock' ||
            // Block quotes
            node.name === 'Blockquote' ||
            // Lists (but not individual list items)
            node.name === 'BulletList' ||
            node.name === 'OrderedList'

          if (shouldFold) {
            const isFoldable = foldable(state, node.from, node.to)
            if (isFoldable) {
              foldRanges.push({ from: isFoldable.from, to: isFoldable.to })
            }
          }
        },
      })

      if (foldRanges.length > 0) {
        view.dispatch({
          effects: foldRanges.map((range) =>
            foldEffect.of({ from: range.from, to: range.to })
          ),
        })
      }
    })

    Vim.defineAction('foldOpenAll', (cm: unknown) => {
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      const view = (cm as any).cm6 || cm
      unfoldAll(view)
    })

    Vim.defineAction('foldMore', (cm: unknown) => {
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      const view = (cm as any).cm6 || cm
      foldCode(view)
    })

    Vim.defineAction('foldLess', (cm: unknown) => {
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      const view = (cm as any).cm6 || cm
      unfoldCode(view)
    })

    // Map vim folding keys to actions
    Vim.mapCommand('zc', 'action', 'foldClose', undefined, {})
    Vim.mapCommand('zo', 'action', 'foldOpen', undefined, {})
    Vim.mapCommand('za', 'action', 'foldToggle', undefined, {})
    Vim.mapCommand('zC', 'action', 'foldClose', undefined, {}) // Close recursively
    Vim.mapCommand('zO', 'action', 'foldOpen', undefined, {}) // Open recursively
    Vim.mapCommand('zA', 'action', 'foldToggle', undefined, {}) // Toggle recursively
    Vim.mapCommand('zM', 'action', 'foldCloseAll', undefined, {})
    Vim.mapCommand('zR', 'action', 'foldOpenAll', undefined, {})
    Vim.mapCommand('zm', 'action', 'foldMore', undefined, {})
    Vim.mapCommand('zr', 'action', 'foldLess', undefined, {})
  }

  function getLanguageExtension(filename: string): Extension {
    if (!filename)
      return markdown({
        base: markdownLanguage,
        codeLanguages: languages,
      })
    const ext = filename.split('.').pop()?.toLowerCase()
    switch (ext) {
      case 'toml':
        return StreamLanguage.define(toml)
      case 'md':
      case 'markdown':
      default:
        return markdown({
          base: markdownLanguage,
          codeLanguages: languages,
        })
    }
  }

  function destroyEditor(): void {
    if (editorView) {
      editorView.destroy()
      editorView = null
    }
  }

  function createCodeMirrorEditor(): void {
    if (!editorContainer) return

    prepareContainer()

    try {
      // Setup vim folding commands if in vim mode
      if (keyBindingMode === 'vim') {
        setupVimFoldingCommands()
      }

      const extensions = buildEditorConfiguration()
      const newEditorView = new EditorView({
        doc: value || '',
        extensions,
        parent: editorContainer,
      })

      editorView = newEditorView
      scrollToHeader()
    } catch (error) {
      handleCreationFailure(error)
    }
  }

  function prepareContainer(): void {
    destroyEditor()
    editorContainer.innerHTML = ''
  }

  function buildEditorConfiguration(): Extension[] {
    const keymaps = createKeymaps()
    const updateListener = EditorView.updateListener.of((update) => {
      if (update.docChanged) {
        const newValue = update.state.doc.toString()
        lastPropsValue = newValue
        onContentChange?.(newValue)
        const isDirty = newValue !== initialValue
        handleDirtyChange(isDirty)
      }
    })

    const extensions: Extension[] = [
      getKeyMappingsMode(keyBindingMode),
      basicSetup,
      getLanguageExtension(filename),
      codeFolding(),
      currentTheme,
      createFontExtension(editorFontFamily, editorFontSize),
      ...keymaps,
      ...(configStateManager.editor.word_wrap ? [EditorView.lineWrapping] : []),
      indentUnit.of(
        configStateManager.editor.expand_tabs
          ? ' '.repeat(configStateManager.editor.tab_size || 2)
          : '\t'
      ),
      EditorState.tabSize.of(configStateManager.editor.tab_size || 2),
      updateListener,
    ].filter((ext): ext is Extension => Boolean(ext))

    return extensions
  }

  function createKeymaps(): Extension[] {
    const insertSpaces = (view: EditorView): boolean => {
      const tabSize = configStateManager.editor.tab_size || 2
      const spaces = ' '.repeat(tabSize)
      const { from, to } = view.state.selection.main
      view.dispatch({
        changes: { from, to, insert: spaces },
        selection: { anchor: from + spaces.length },
      })
      return true
    }

    const tabBinding = configStateManager.editor.expand_tabs
      ? { key: 'Tab', run: insertSpaces }
      : indentWithTab

    const customKeymap = keymap.of([
      tabBinding,
      {
        key: 'Ctrl-s',
        run: (): boolean => {
          onSave()
          resetDirtyFlag()
          return true
        },
      },
      // Folding shortcuts for all modes
      { key: 'Ctrl-Shift-[', run: foldCode },
      { key: 'Ctrl-Shift-]', run: unfoldCode },
      { key: 'Ctrl-Alt-[', run: foldAll },
      { key: 'Ctrl-Alt-]', run: unfoldAll },
    ])

    const escapeKeymap =
      onExit || onRequestExit
        ? keymap.of([
            {
              key: 'Escape',
              run: (): boolean => {
                setTimeout(() => {
                  try {
                    if (keyBindingMode === 'vim') {
                      return false
                    }

                    const isDirty = value !== initialValue
                    if (isDirty && onRequestExit) {
                      onRequestExit()
                    } else if (onExit) {
                      onExit()
                    }
                  } catch {
                    if (onExit) onExit()
                  }
                }, 100)
                return false
              },
            },
          ])
        : null

    return [customKeymap, escapeKeymap].filter(Boolean) as Extension[]
  }

  function handleCreationFailure(error: unknown): void {
    console.error('Failed to create CodeMirror editor:', error)
    createFallbackEditor()
  }

  function scrollToHeader(): void {
    if (nearestHeaderText.length > 2 && editorView) {
      setTimeout(() => {
        if (editorView) {
          const doc = editorView.state.doc
          const fullText = doc.toString()

          function escapeRegex(text: string): string {
            return text.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
          }

          const headerRegex = new RegExp(
            `^#+\\s*${escapeRegex(nearestHeaderText)}\\s*$`,
            'm'
          )
          const match = fullText.match(headerRegex)

          if (match && match.index !== undefined) {
            editorView.dispatch({
              selection: { anchor: match.index, head: match.index },
              effects: EditorView.scrollIntoView(match.index, {
                y: 'start',
                yMargin: 80,
              }),
            })
          }

          editorView.focus()
        }
      }, 150)
    } else {
      setTimeout(() => {
        if (editorView) {
          editorView.focus()
        }
      }, 100)
    }
  }

  let fallbackInputHandler: ((event: Event) => void) | null = null

  function createFallbackEditor(): void {
    if (!editorContainer) return
    editorContainer.innerHTML =
      '<textarea style="width:100%; height:100%; background:#282828; color:#fbf1c7; font-family:\'JetBrains Mono\', monospace; padding:16px; border:none; resize:none;"></textarea>'
    const textarea = editorContainer.querySelector(
      'textarea'
    ) as HTMLTextAreaElement
    if (textarea) {
      textarea.value = value || ''

      if (fallbackInputHandler) {
        textarea.removeEventListener('input', fallbackInputHandler)
      }

      fallbackInputHandler = () => {
        onContentChange?.(textarea.value)
      }

      textarea.addEventListener('input', fallbackInputHandler)
      setTimeout(() => textarea.focus(), 10)
    }
  }

  onMount(() => {
    const init = async () => {
      await tick()
      // Create editor with current reactive config values
      createCodeMirrorEditor()
    }

    init()

    return () => {
      if (editorView) {
        editorView.destroy()
        editorView = null
      }

      if (fallbackInputHandler && editorContainer) {
        const textarea = editorContainer.querySelector('textarea')
        if (textarea) {
          textarea.removeEventListener('input', fallbackInputHandler)
        }
        fallbackInputHandler = null
      }
    }
  })
</script>

<div class="editor-container">
  <div bind:this={editorContainer} class="codemirror-editor"></div>
  <div class="edit-footer">
    <h3>Editing: {filename}</h3>
    <div class="edit-controls">
      <button onclick={onSave} class="save-btn">Save (Ctrl+S)</button>
      <button onclick={onExit} class="cancel-btn">Cancel (Esc)</button>
    </div>
  </div>
</div>

<style>
  .codemirror-editor {
    flex: 1;
    min-height: 0;
    background-color: #282828;
    overflow: hidden;
    display: flex;
    flex-direction: column;
  }

  .codemirror-editor :global(.cm-editor) {
    height: 100% !important;
  }

  .codemirror-editor :global(.cm-scroller) {
    height: 100% !important;
    overflow-y: auto !important;
  }

  .codemirror-editor :global(.cm-editor) {
    margin-left: max(1em, calc((100vw - 100ch) / 2)) !important;
    margin-right: max(1em, calc((100vw - 100ch) / 2)) !important;
  }

  @media (min-width: 768px) {
    .codemirror-editor :global(.cm-editor) {
      margin-left: max(1.5em, calc((100vw - 110ch) / 2)) !important;
      margin-right: max(1.5em, calc((100vw - 110ch) / 2)) !important;
    }
  }

  @media (min-width: 1024px) {
    .codemirror-editor :global(.cm-editor) {
      margin-left: max(2em, calc((100vw - 120ch) / 2)) !important;
      margin-right: max(2em, calc((100vw - 120ch) / 2)) !important;
    }
  }

  /* Add padding specifically to the text content, not the gutter */
  .codemirror-editor :global(.cm-content) {
    padding-top: 1.2em !important;
    padding-left: 1em !important;
    padding-right: 1em !important;
  }

  /* Editor container styles */
  .editor-container {
    flex: 1;
    display: flex;
    flex-direction: column;
    background-color: #21252b;
    min-height: 0;
  }

  /* Edit footer styles with responsive width to match editor */
  .edit-footer {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.8em 0;
    border-top: 1px solid #181a1f;
    background-color: #21252b;
    flex-shrink: 0;
    margin-left: max(1em, calc((100vw - 100ch) / 2));
    margin-right: max(1em, calc((100vw - 100ch) / 2));
    padding-left: 1em;
    padding-right: 1em;
  }

  @media (min-width: 768px) {
    .edit-footer {
      margin-left: max(1.5em, calc((100vw - 110ch) / 2));
      margin-right: max(1.5em, calc((100vw - 110ch) / 2));
    }
  }

  @media (min-width: 1024px) {
    .edit-footer {
      margin-left: max(2em, calc((100vw - 120ch) / 2));
      margin-right: max(2em, calc((100vw - 120ch) / 2));
    }
  }

  .edit-footer h3 {
    margin: 0;
    color: #61afef;
    font-size: 1.1em;
    font-weight: 500;
  }

  .edit-controls {
    display: flex;
    gap: 0.5em;
  }

  .save-btn,
  .cancel-btn {
    padding: 0.4em 0.8em;
    border: none;
    border-radius: 4px;
    font-size: 0.9em;
    cursor: pointer;
    transition: background-color 0.2s ease;
  }

  .save-btn {
    background-color: #98c379;
    color: #282c34;
  }

  .save-btn:hover {
    background-color: #a7d78b;
  }

  .cancel-btn {
    background-color: #3a3f4b;
    color: #abb2bf;
  }

  .cancel-btn:hover {
    background-color: #4b5263;
  }

  .codemirror-editor :global(.cm-foldPlaceholder) {
    background: linear-gradient(135deg, #3c3836 0%, #504945 100%) !important;
    color: #d5c4a1 !important;
    border: 1px solid #665c54 !important;
    border-radius: 6px !important;
    padding: 2px 8px !important;
    font-size: 11px !important;
    font-weight: 500 !important;
    margin: 0 4px !important;
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.2) !important;
  }
</style>
