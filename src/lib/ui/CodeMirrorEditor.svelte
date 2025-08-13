<!--
UI Layer - CodeMirror Editor Core
Focused component handling CodeMirror initialization and content editing.
-->

<script lang="ts">
  import { onMount, tick } from 'svelte'
  import { invoke } from '@tauri-apps/api/core'
  import { EditorView, basicSetup } from 'codemirror'
  import type { Extension } from '@codemirror/state'
  import { keymap } from '@codemirror/view'
  import { indentWithTab } from '@codemirror/commands'
  import { markdown, markdownLanguage } from '@codemirror/lang-markdown'
  import { languages } from '@codemirror/language-data'
  import { StreamLanguage } from '@codemirror/language'
  import { toml } from '@codemirror/legacy-modes/mode/toml'
  import { vim } from '@replit/codemirror-vim'
  import { emacs } from '@replit/codemirror-emacs'
  import { gruvboxDark } from '@fsegurai/codemirror-theme-bundle'

  // TODO: Add theme selection to config options using https://fsegurai.github.io/codemirror-themes/#demo-application

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

  let editorContainer: HTMLElement
  let editorView: EditorView | null = null
  let initialValue = $state(value)
  let lastPropsValue = $state(value)
  let keyBindingMode = $state('basic')

  const propsChanged = $derived(value !== lastPropsValue)

  async function loadEditorMode(): Promise<void> {
    try {
      const mode = await invoke<string>('get_editor_mode')
      keyBindingMode = mode
    } catch (e) {
      console.error('Failed to load editor mode:', e)
      keyBindingMode = 'basic'
    }
  }

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

  function getKeyMappingsMode(mode: string): Extension | null {
    switch (mode) {
      case 'vim':
        return vim()
      case 'emacs':
        return emacs()
      case 'basic':
        return null
      default:
        return null
    }
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
      gruvboxDark,
      ...keymaps,
      EditorView.lineWrapping,
      updateListener,
    ].filter((ext): ext is Extension => Boolean(ext))

    return extensions
  }

  function createKeymaps(): Extension[] {
    const customKeymap = keymap.of([
      indentWithTab,
      {
        key: 'Ctrl-s',
        run: (): boolean => {
          onSave()
          resetDirtyFlag()
          return true
        },
      },
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

  let initialModeSet = $state(false)

  $effect(() => {
    // Only recreate editor when keyBindingMode changes from initial 'basic'
    // This handles the async mode loading from EditorModeManager
    if (!initialModeSet && keyBindingMode !== 'basic') {
      initialModeSet = true
      createCodeMirrorEditor()
    }
  })

  onMount(() => {
    const init = async () => {
      await tick()
      await loadEditorMode()
      // Create editor immediately if mode is already loaded, or with basic mode
      if (keyBindingMode !== 'basic') {
        initialModeSet = true
      }
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

<div bind:this={editorContainer} class="codemirror-editor"></div>

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
</style>
