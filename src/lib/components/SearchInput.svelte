<script lang="ts">
  import { getAppContext } from '../context/app.svelte';
  const context = getAppContext();
</script>

<input
  type="text"
  bind:value={context.state.searchInput}
  placeholder="Search notes... (Enter: edit, Ctrl+enter: new, Ctrl+u/d: scroll)"
  class="search-input"
  bind:this={context.state.searchElement}
  on:focus={() => context.state.isSearchInputFocused = true}
  on:blur={() => context.state.isSearchInputFocused = false}
/>

<style>
.search-input {
  background-color: #3c3836;
  color: #ebdbb2;
  border: 1px solid #504945;
  border-radius: 8px;
  font-size: 1.2em;
  padding: 0.6em;
  margin: 0.5em;
  flex-shrink: 0;
  transition: border-color 0.2s ease, box-shadow 0.2s ease;
}
.search-input:focus {
  outline: none;
  border-color: #83a598;
  box-shadow: 0 0 0 2px rgba(131, 165, 152, 0.2);
}
</style>
