<script lang="ts">
  import { onMount, tick } from 'svelte';
  import { invoke } from "@tauri-apps/api/core";
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
    onSave: () => void;
    onExit?: (() => void) | null;
    onRequestExit?: (() => void) | null;
    onContentChange?: ((newValue: string) => void) | null;
    nearestHeaderText?: string;
    isDirty?: boolean;
  }

  let {
    value = $bindable(),
    filename,
    onSave,
    onExit = null,
    onRequestExit = null,
    onContentChange = null,
    nearestHeaderText = '',
    isDirty = $bindable(false)
  }: Props = $props();

  // Track the initial value when props change
  let initialValue = $state(value);
  let lastPropsValue = $state(value);

  // Derived value to detect when external props change
  const propsChanged = $derived(value !== lastPropsValue);

  // Update tracking values when props change
  $effect.pre(() => {
    if (propsChanged) {
      initialValue = value;
      lastPropsValue = value;
      isDirty = false;
    }
  });

  function resetDirtyFlag(): void {
    isDirty = false;
    initialValue = value;
    lastPropsValue = value;
  }


  let editorContainer: HTMLElement;
  let editorView: EditorView | null;
  let keyBindingMode: string = 'basic';

  async function loadEditorMode(): Promise<void> {
    try {
      keyBindingMode = await invoke<string>("get_editor_mode");
    } catch (e) {
      console.error("Failed to load editor mode:", e);
      keyBindingMode = 'basic';
    }
  }

  function getKeyMappingsMode(mode: string): any {
    switch (mode) {
      case 'vim': return vim();
      case 'emacs': return emacs();
      case 'basic': return null;
      default: return null;
    }
  }

  function createFallbackEditor(): void {
    if (!editorContainer) return;
    editorContainer.innerHTML = '<textarea style="width:100%; height:100%; background:#282828; color:#fbf1c7; font-family:\'JetBrains Mono\', monospace; padding:16px; border:none; resize:none;"></textarea>';
    const textarea = editorContainer.querySelector('textarea') as HTMLTextAreaElement;
    if (textarea) {
      textarea.value = value || '';
      textarea.addEventListener('input', () => {
        value = textarea.value;
        onContentChange?.(textarea.value);
      });
      setTimeout(() => textarea.focus(), 10);
    }
  }

  function createCodeMirrorEditor(): void {
    if (!editorContainer) {
      console.error('Edit container not found');
      return;
    }
    if (editorView) {
      editorView.destroy();
      editorView = null;
    }
    editorContainer.innerHTML = '';

    try {
      const gruvboxTheme = EditorView.theme({
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

      const gruvboxHighlighting = syntaxHighlighting(HighlightStyle.define([
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

      function getLanguageExtension(filename: string): any {
        if (!filename) return markdown();
        const ext = filename.split('.').pop()?.toLowerCase();
        switch (ext) {
          //case 'js': case 'jsx': case 'ts': case 'tsx': return javascript();
          //case 'rs': return rust();
          //case 'html': case 'htm': return html();
          //case 'css': return css();
          case 'toml': return StreamLanguage.define(toml);
          case 'md': case 'markdown': default: return markdown();
        }
      }

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
              // In vim mode, let vim handle escape first
              if (keyBindingMode === 'vim') {
                // Don't exit if we might be in vim insert mode
                // This is a simplified check - in practice vim will handle escape
                return false;
              }

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

      const keyMappingsMode = getKeyMappingsMode(keyBindingMode);

      const extensions: any[] = [
        keyMappingsMode,
        basicSetup,
        getLanguageExtension(filename),
        gruvboxTheme,
        gruvboxHighlighting,
        customKeymap,
        escapeKeymap,
        EditorView.lineWrapping,
        EditorView.updateListener.of((update) => {
          if (update.docChanged) {
            const newValue = update.state.doc.toString();
            value = newValue;
            lastPropsValue = newValue;
            onContentChange?.(newValue);
            if (!isDirty && newValue !== initialValue) {
              isDirty = true;
            }
          }
        })
      ].filter((ext): ext is any => Boolean(ext));

      editorView = new EditorView({
        doc: value || '',
        extensions,
        parent: editorContainer
      });

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
            } else {
              console.log('Header not found in markdown');
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

    } catch (error) {
      console.error('Failed to create CodeMirror editor:', error);
      createFallbackEditor();
    }
  }

  onMount(() => {
    const init = async () => {
      await tick();
      await loadEditorMode();
      createCodeMirrorEditor();
    };

    init();

    return () => {
      if (editorView) {
        editorView.destroy();
      }
    };
  });

</script>

<div bind:this={editorContainer} class="editor-container"></div>

<style>
.editor-container {
  flex: 1;
  height: 100%;
  background-color: #282828;
  border: 2px solid transparent;
  transition: border-color 0.2s ease;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}
.editor-container:focus-within {
  border-color: #83a598;
}
.editor-container :global(.cm-editor) {
  height: 100% !important;
}
.editor-container :global(.cm-scroller) {
  height: 100% !important;
  overflow-y: auto !important;
}
.editor-container::-webkit-scrollbar {
  width: 8px;
}
.editor-container::-webkit-scrollbar-track {
  background: #282828;
}
.editor-container::-webkit-scrollbar-thumb {
  background: #504945;
  border-radius: 4px;
}
.editor-container::-webkit-scrollbar-thumb:hover {
  background: #665c54;
}
</style>
