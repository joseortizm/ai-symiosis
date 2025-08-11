<!--
UI Layer - Search Input
Search input field with debounced queries and focus state tracking.
Connects to search manager and handles keyboard navigation events.
-->

<script lang="ts">
  import { getContext } from 'svelte';

  const { searchManager, appCoordinator } = getContext('managers') as any;

  let searchElement: HTMLInputElement;

  $effect(() => {
    if (searchElement) {
      appCoordinator.context.focusManager.setSearchElement(searchElement);
    }
  });
</script>

<input
  type="text"
  bind:value={searchManager.searchInput}
  placeholder="Search notes... (Enter: edit, Ctrl+enter: new, Ctrl+u/d: scroll)"
  class="search-input"
  bind:this={searchElement}
  onfocus={() => appCoordinator.context.focusManager.setSearchInputFocused(true)}
  onblur={() => appCoordinator.context.focusManager.setSearchInputFocused(false)}
/>

<style>
.search-input {
  background-color: #3c3836;
  color: #ebdbb2;
  border: 1px solid #504945;
  border-radius: 8px;
  font-size: 1.3em;
  padding: 0.6em;
  margin: 0.5em;
  flex-shrink: 0;
  transition: border-color 0.2s ease, box-shadow 0.2s ease;
}
.search-input::placeholder {
  color: light-gray;
  font-size: 0.8em;
  opacity: 0.5;
}
.search-input:focus {
  outline: none;
  border-color: #83a598;
  box-shadow: 0 0 0 2px rgba(131, 165, 152, 0.2);
}
</style>
