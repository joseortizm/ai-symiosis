<script>
  import { onMount, tick } from 'svelte';
  import { EditorView, basicSetup } from 'codemirror';
  import { keymap } from '@codemirror/view';
  import { indentWithTab } from '@codemirror/commands';
  import { markdown } from '@codemirror/lang-markdown';
  import { javascript } from '@codemirror/lang-javascript';
  import { python } from '@codemirror/lang-python';
  import { rust } from '@codemirror/lang-rust';
  import { html } from '@codemirror/lang-html';
  import { css } from '@codemirror/lang-css';
  import { json } from '@codemirror/lang-json';
  import { xml } from '@codemirror/lang-xml';
  import { sql } from '@codemirror/lang-sql';
  import { syntaxHighlighting, HighlightStyle } from '@codemirror/language';
  import { tags } from '@lezer/highlight';
  import { vim } from "@replit/codemirror-vim";

  export let value;
  export let filename;
  export let onSave;
  // export let onExit;

  let container;
  let editorView;

  function createFallbackEditor() {
    if (!container) return;
    container.innerHTML = '<textarea style="width:100%; height:100%; background:#282828; color:#fbf1c7; font-family:\'JetBrains Mono\', monospace; padding:16px; border:none; resize:none;"></textarea>';
    const textarea = container.querySelector('textarea');
    if (textarea) {
      textarea.value = value || '';
      textarea.addEventListener('input', () => {
        value = textarea.value;
      });
      setTimeout(() => textarea.focus(), 10);
    }
  }

  function createCodeMirrorEditor() {
    if (!container) {
      console.error('Edit container not found');
      return;
    }
    if (editorView) {
      editorView.destroy();
      editorView = null;
    }
    container.innerHTML = '';

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

      function getLanguageExtension(filename) {
        if (!filename) return markdown();
        const ext = filename.split('.').pop()?.toLowerCase();
        switch (ext) {
          case 'js': case 'jsx': case 'ts': case 'tsx': return javascript();
          case 'py': return python();
          case 'rs': return rust();
          case 'html': case 'htm': return html();
          case 'css': return css();
          case 'json': return json();
          case 'xml': return xml();
          case 'sql': return sql();
          case 'md': case 'markdown': default: return markdown();
        }
      }

      const customKeymap = keymap.of([
        indentWithTab,
        { key: "Ctrl-s", run: () => { onSave(); return true; } },
      ]);

      const extensions = [
        vim(), // Vim keymap
        basicSetup,
        getLanguageExtension(filename),
        gruvboxTheme,
        gruvboxHighlighting,
        customKeymap,
        EditorView.lineWrapping,
        EditorView.updateListener.of((update) => {
          if (update.docChanged) {
            value = update.state.doc.toString();
          }
        })
      ].filter(Boolean);

      editorView = new EditorView({
        doc: value || '',
        extensions,
        parent: container
      });

      setTimeout(() => {
        if (editorView) {
          editorView.focus();
        }
      }, 100);

    } catch (error) {
      console.error('Failed to create CodeMirror editor:', error);
      createFallbackEditor();
    }
  }

  onMount(async () => {
    await tick();
    createCodeMirrorEditor();
    return () => {
      if (editorView) {
        editorView.destroy();
      }
    };
  });

</script>

<div bind:this={container} class="editor-container"></div>

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
