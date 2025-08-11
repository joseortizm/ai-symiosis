<!--
UI Layer - CodeMirror Editor Core
Focused component handling CodeMirror initialization and content editing.
-->

<script lang="ts">
  import { onMount, tick } from 'svelte';
  import { EditorView, basicSetup } from 'codemirror';
  import { keymap } from '@codemirror/view';
  import { indentWithTab } from '@codemirror/commands';
  import { markdown } from '@codemirror/lang-markdown';
  import { syntaxHighlighting, HighlightStyle } from '@codemirror/language';
  import { tags } from '@lezer/highlight';
  import { StreamLanguage } from '@codemirror/language';
  import { toml } from '@codemirror/legacy-modes/mode/toml';
  import { vim } from "@replit/codemirror-vim";
  import { emacs } from "@replit/codemirror-emacs";

  interface Props {
    value: string;
    filename: string;
    keyBindingMode: string;
    nearestHeaderText?: string;
    onSave: () => void;
    onContentChange?: (newValue: string) => void;
    onDirtyChange?: (isDirty: boolean) => void;
    onExit?: (() => void) | null | undefined;
    onRequestExit?: (() => void) | null | undefined;
  }

  let {
    value = $bindable(),
    filename,
    keyBindingMode,
    nearestHeaderText = '',
    onSave,
    onContentChange,
    onDirtyChange,
    onExit = null,
    onRequestExit = null
  }: Props = $props();

  let editorContainer: HTMLElement;
  let editorView: EditorView | null = null;
  let initialValue = $state(value);
  let lastPropsValue = $state(value);

  const propsChanged = $derived(value !== lastPropsValue);

  $effect.pre(() => {
    if (propsChanged) {
      initialValue = value;
      lastPropsValue = value;
      onDirtyChange?.(false);
    }
  });

  function resetDirtyFlag(): void {
    onDirtyChange?.(false);
    initialValue = value;
    lastPropsValue = value;
  }

  function getKeyMappingsMode(mode: string): any {
    switch (mode) {
      case 'vim': return vim();
      case 'emacs': return emacs();
      case 'basic': return null;
      default: return null;
    }
  }

  function getLanguageExtension(filename: string): any {
    if (!filename) return markdown();
    const ext = filename.split('.').pop()?.toLowerCase();
    switch (ext) {
      case 'toml': return StreamLanguage.define(toml);
      case 'md': case 'markdown': default: return markdown();
    }
  }

  function destroyEditor(): void {
    if (editorView) {
      editorView.destroy();
      editorView = null;
    }
  }

  function createGruvboxTheme(): any {
    return EditorView.theme({
      "&": {
        color: "#fbf1c7",
        backgroundColor: "#282828",
        height: "100%",
        fontSize: "14px"
      },
      ".cm-content": {
        padding: "16px",
        minHeight: "100%",
        caretColor: "#fbf1c7",
        fontFamily: "'JetBrains Mono', 'Consolas', monospace",
        fontSize: "14px",
        lineHeight: "1.5"
      },
      ".cm-focused": {
        outline: "none"
      },
      ".cm-editor": {
        height: "100%"
      },
      ".cm-scroller": {
        fontFamily: "'JetBrains Mono', 'Consolas', monospace",
        height: "100%"
      },
      ".cm-cursor": {
        borderColor: "#fbf1c7"
      },
      ".cm-selectionBackground": {
        backgroundColor: "#504945 !important"
      },
      ".cm-focused .cm-selectionBackground": {
        backgroundColor: "#504945 !important"
      },
      ".cm-activeLine": {
        backgroundColor: "#32302f"
      },
      ".cm-activeLineGutter": {
        backgroundColor: "#32302f"
      },
      ".cm-gutters": {
        backgroundColor: "#32302f",
        color: "#a89984",
        border: "none"
      },
      ".cm-lineNumbers": {
        color: "#a89984"
      },
      ".cm-fat-cursor": {
        backgroundColor: "#fe8019 !important"
      },
      ".cm-animate-fat-cursor": {
        backgroundColor: "#fe8019 !important"
      }
    });
  }

  function createGruvboxHighlighting(): any {
    return syntaxHighlighting(HighlightStyle.define([
      { tag: tags.comment, color: "#928374", fontStyle: "italic" },
      { tag: tags.keyword, color: "#fb4934" },
      { tag: tags.string, color: "#b8bb26" },
      { tag: tags.number, color: "#d3869b" },
      { tag: tags.function(tags.variableName), color: "#8ec07c" },
      { tag: tags.variableName, color: "#fbf1c7" },
      { tag: tags.propertyName, color: "#83a598" },
      { tag: tags.typeName, color: "#fabd2f" },
      { tag: tags.operator, color: "#fe8019" },
      { tag: tags.punctuation, color: "#fbf1c7" },
      { tag: tags.heading1, color: "#fb4934", fontWeight: "bold", fontSize: "1.6em" },
      { tag: tags.heading2, color: "#fabd2f", fontWeight: "bold", fontSize: "1.4em" },
      { tag: tags.heading3, color: "#b8bb26", fontWeight: "bold", fontSize: "1.2em" },
      { tag: tags.strong, color: "#fe8019", fontWeight: "bold" },
      { tag: tags.emphasis, color: "#d3869b", fontStyle: "italic" },
      { tag: tags.link, color: "#83a598", textDecoration: "underline" },
      { tag: tags.monospace, color: "#d3869b", backgroundColor: "#3c3836" }
    ]));
  }

  function createCodeMirrorEditor(): void {
    if (!editorContainer) return;

    destroyEditor();
    editorContainer.innerHTML = '';

    try {
      const customKeymap = keymap.of([
        indentWithTab,
        { key: "Ctrl-s", run: (): boolean => {
          onSave();
          resetDirtyFlag();
          return true;
        } },
      ]);

      const escapeKeymap = (onExit || onRequestExit) ? keymap.of([{
        key: "Escape",
        run: (): boolean => {
          setTimeout(() => {
            try {
              if (keyBindingMode === 'vim') {
                return false;
              }

              const isDirty = value !== initialValue;
              if (isDirty && onRequestExit) {
                onRequestExit();
              } else if (onExit) {
                onExit();
              }
            } catch (e) {
              if (onExit) onExit();
            }
          }, 100);
          return false;
        }
      }]) : null;

      const extensions: any[] = [
        getKeyMappingsMode(keyBindingMode),
        basicSetup,
        getLanguageExtension(filename),
        createGruvboxTheme(),
        createGruvboxHighlighting(),
        customKeymap,
        escapeKeymap,
        EditorView.lineWrapping,
        EditorView.updateListener.of((update) => {
          if (update.docChanged) {
            const newValue = update.state.doc.toString();
            value = newValue;
            lastPropsValue = newValue;
            onContentChange?.(newValue);
            const isDirty = newValue !== initialValue;
            onDirtyChange?.(isDirty);
          }
        })
      ].filter((ext): ext is any => Boolean(ext));

      const newEditorView = new EditorView({
        doc: value || '',
        extensions,
        parent: editorContainer
      });

      editorView = newEditorView;
      scrollToHeader();
    } catch (error) {
      console.error('Failed to create CodeMirror editor:', error);
      createFallbackEditor();
    }
  }

  function scrollToHeader(): void {
    if (nearestHeaderText.length > 2 && editorView) {
      setTimeout(() => {
        if (editorView) {
          const doc = editorView.state.doc;
          const fullText = doc.toString();

          function escapeRegex(text: string): string {
            return text.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
          }

          const headerRegex = new RegExp(`^#+\\s*${escapeRegex(nearestHeaderText)}\\s*$`, 'm');
          const match = fullText.match(headerRegex);

          if (match && match.index !== undefined) {
            editorView.dispatch({
              selection: { anchor: match.index, head: match.index },
              effects: EditorView.scrollIntoView(match.index, { y: "start", yMargin: 80 })
            });
          }

          editorView.focus();
        }
      }, 150);
    } else {
      setTimeout(() => {
        if (editorView) {
          editorView.focus();
        }
      }, 100);
    }
  }

  let fallbackInputHandler: ((event: Event) => void) | null = null;

  function createFallbackEditor(): void {
    if (!editorContainer) return;
    editorContainer.innerHTML = '<textarea style="width:100%; height:100%; background:#282828; color:#fbf1c7; font-family:\'JetBrains Mono\', monospace; padding:16px; border:none; resize:none;"></textarea>';
    const textarea = editorContainer.querySelector('textarea') as HTMLTextAreaElement;
    if (textarea) {
      textarea.value = value || '';

      if (fallbackInputHandler) {
        textarea.removeEventListener('input', fallbackInputHandler);
      }

      fallbackInputHandler = () => {
        value = textarea.value;
        onContentChange?.(textarea.value);
      };

      textarea.addEventListener('input', fallbackInputHandler);
      setTimeout(() => textarea.focus(), 10);
    }
  }

  $effect(() => {
    keyBindingMode;
    if (editorView) {
      createCodeMirrorEditor();
    }
  });

  onMount(() => {
    const init = async () => {
      await tick();
      createCodeMirrorEditor();
    };

    init();

    return () => {
      if (editorView) {
        editorView.destroy();
        editorView = null;
      }

      if (fallbackInputHandler && editorContainer) {
        const textarea = editorContainer.querySelector('textarea');
        if (textarea) {
          textarea.removeEventListener('input', fallbackInputHandler);
        }
        fallbackInputHandler = null;
      }
    };
  });


</script>

<div bind:this={editorContainer} class="codemirror-editor"></div>

<style>
.codemirror-editor {
  flex: 1;
  height: 100%;
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
</style>
