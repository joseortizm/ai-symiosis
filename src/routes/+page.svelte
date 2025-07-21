<script>
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";

  let notes = $state([]);
  let selectedNote = $state(null);
  let selectedIndex = $state(-1);
  let searchInput = $state('');

  onMount(async () => {
    try {
      const fetchedNotes = await invoke("list_notes");
      notes = fetchedNotes;
      if (notes.length > 0) {
        selectedIndex = 0;
      }
    } catch (e) {
      console.error(e);
    }
  });

  $effect(() => {
    if (notes.length > 0 && selectedIndex !== -1) {
      selectedNote = notes[selectedIndex];
    } else {
      selectedNote = null;
    }
  });

  function selectNote(note, index) {
    selectedIndex = index;
  }

  function handleKeydown(event) {
    if (event.target.tagName === 'INPUT') return;
    if (notes.length === 0) return;

    switch (event.key) {
      case 'ArrowUp':
      case 'k':
        event.preventDefault();
        selectedIndex = Math.max(0, selectedIndex - 1);
        break;
      case 'ArrowDown':
      case 'j':
        event.preventDefault();
        selectedIndex = Math.min(notes.length - 1, selectedIndex + 1);
        break;
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<main class="container">
  <input type="text" bind:value={searchInput} placeholder="Search..." class="search-input">
  <ul>
    {#each notes as note, index}
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
    text-align: center;
  }
  .search-input {
    padding: 0.5em;
    margin: 0.5em;
    border: 1px solid #ccc;
    border-radius: 4px;
  }
  ul {
    list-style: none;
    padding: 0;
    margin: 0;
    overflow-y: auto; /* Make list scrollable */
    flex-grow: 1;
  }
  li {
    margin: 0;
  }
  button {
    width: 100%;
    padding: 0.5em;
    cursor: pointer;
    border: none;
    border-bottom: 1px solid #eee;
    background: none;
    text-align: left;
  }
  .selected {
    background-color: #ddd;
  }
</style>

