<!--
UI Layer - App Layout
Main container with slots for search, list, view, and modal components.
Defines the overall application structure and CSS grid layout.
-->

<main class="container">
  <slot name="search" />
  <slot name="list" />
  <slot name="view" />
  <slot name="modals" />
</main>

<style>
  :global(body) {
    margin: 0;
    background-color: #282c34;
  }

  .container {
    margin: 0;
    display: flex;
    flex-direction: column;
    height: 100vh;
    background-color: #282828;
    color: #ebdbb2;
    font-family: 'Inter', sans-serif;
  }
</style>
