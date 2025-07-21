<script>
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";

  let notes = $state([]);
  let selectedNote = $state(null);

  onMount(async () => {
    try {
      notes = await invoke("list_notes");
    } catch (e) {
      console.error(e);
    }
  });

  function selectNote(note) {
    selectedNote = note;
  }
</script>

<main class="container">
  <h1>Notes</h1>
  <ul>
    {#each notes as note}
      <li>
        <button class:selected={note === selectedNote} onclick={() => selectNote(note)}>
          {note}
        </button>
      </li>
    {/each}
  </ul>
</main>

<style>
  .container {
    margin: 0;
    padding-top: 10vh;
    display: flex;
    flex-direction: column;
    justify-content: center;
    text-align: center;
  }
  ul {
    list-style: none;
    padding: 0;
  }
  li {
    margin: 0.2em 0;
  }
  button {
    width: 100%;
    padding: 0.5em;
    cursor: pointer;
    border: 1px solid #ccc;
    background: none;
    text-align: left;
  }
  .selected {
    background-color: #eee;
  }
</style>

