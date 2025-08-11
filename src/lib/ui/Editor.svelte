<!--
UI Layer - Editor Container
Main editor component that orchestrates focused sub-components.
-->

<script lang="ts">
  import CodeMirrorEditor from './CodeMirrorEditor.svelte';
  import EditorModeManager from './EditorModeManager.svelte';

  interface Props {
    value: string;
    filename: string;
    onSave: () => void;
    onExit?: (() => void) | undefined;
    onRequestExit?: (() => void) | undefined;
    onContentChange?: ((newValue: string) => void) | undefined;
    nearestHeaderText?: string;
    isDirty?: boolean;
  }

  let {
    value = $bindable(),
    filename,
    onSave,
    onExit = undefined,
    onRequestExit = undefined,
    onContentChange = undefined,
    nearestHeaderText = '',
    isDirty = $bindable(false)
  }: Props = $props();

  let keyBindingMode = $state('basic');

  function handleModeLoaded(mode: string): void {
    keyBindingMode = mode;
  }

  function handleDirtyChange(dirty: boolean): void {
    isDirty = dirty;
  }

</script>

<EditorModeManager onModeLoaded={handleModeLoaded} />
<div class="editor-container">
  <CodeMirrorEditor 
    bind:value
    {filename}
    {keyBindingMode}
    {nearestHeaderText}
    {onSave}
    {onExit}
    {onRequestExit}
    onContentChange={onContentChange}
    onDirtyChange={handleDirtyChange}
  />
</div>

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
