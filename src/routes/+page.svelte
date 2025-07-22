<script>
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";

  let filteredNotes = $state([]);
  let selectedNote = $state(null);
  let selectedIndex = $state(-1);
  let searchInput = $state('');
  let searchElement;
  let noteListElement; // Reference to the ul element
  let isSearchInputFocused = $state(false); // New state variable

  onMount(() => {
    searchElement.focus();
  });

  $effect(async () => {
    try {
      filteredNotes = await invoke("list_notes", { query: searchInput });
      if (filteredNotes.length === 0) {
        selectedIndex = -1;
      } else if (selectedIndex >= filteredNotes.length) {
        selectedIndex = filteredNotes.length - 1;
      } else if (selectedIndex === -1 && filteredNotes.length > 0) {
        selectedIndex = 0;
      }
    } catch (e) {
      console.error(e);
    }
  });

  $effect(() => {
    if (filteredNotes.length > 0 && selectedIndex !== -1) {
      selectedNote = filteredNotes[selectedIndex];
    } else {
      selectedNote = null;
    }
  });

  function selectNote(note, index) {
    selectedIndex = index;
  }

  function handleKeydown(event) {
    if (isSearchInputFocused) {
      if (event.key === 'Enter') {
        event.preventDefault();
        if (filteredNotes.length > 0) {
          selectedIndex = 0; // Select the first item
          noteListElement.focus(); // Focus the list element
        }
      }
      return; // Do not process other shortcuts when search is focused
    }

    if (filteredNotes.length === 0) return;

    switch (event.key) {
      case 'ArrowUp':
      case 'k':
        event.preventDefault();
        selectedIndex = Math.max(0, selectedIndex - 1);
        break;
      case 'ArrowDown':
      case 'j':
        event.preventDefault();
        selectedIndex = Math.min(filteredNotes.length - 1, selectedIndex + 1);
        break;
      case 'Enter':
        if (selectedNote) {
          invoke('open_note', { noteName: selectedNote });
        }
        break;
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<main class="container">
  <input type="text" bind:value={searchInput} placeholder="Search..." class="search-input" bind:this={searchElement} onfocus={() => isSearchInputFocused = true} onblur={() => isSearchInputFocused = false}>
  <ul bind:this={noteListElement} tabindex="-1">
    {#each filteredNotes as note, index}
      <li>
        <button class:selected={note === selectedNote} onclick={() => selectNote(note, index)}>
          {note}
        </button>
      </li>
    {/each}
  </ul>
</main>

<style>
  .container {
    margin: 0;
    display: flex;
    flex-direction: column;
    height: 100vh;
    background-color: #282828;
    color: #ebdbb2;
    font-family: 'Inter', sans-serif;
  }
  .search-input {
    background-color: #3c3836;
    color: #ebdbb2;
    border: 1px solid #504945;
    border-radius: 8px;
    font-size: 1.5em;
    padding: 0.8em;
    margin: 0.5em;
  }
  .search-input:focus {
    outline: none;
    border-color: #83a598;
  }
  ul {
    list-style: none;
    padding: 0;
    margin: 0;
    overflow-y: auto;
    flex-grow: 1;
  }
  li {
    margin: 0;
  }
  button {
    width: 100%;
    padding: 0.8em 1.2em;
    cursor: pointer;
    border: none;
    border-bottom: 1px solid #3c3836;
    background: none;
    color: #ebdbb2;
    text-align: left;
    font-size: 1em;
  }
  .selected {
    background-color: #504945;
    color: #fe8019;
  }
</style>

