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
    background-color: var(--theme-bg-secondary);
  }

  .container {
    margin: 0;
    display: flex;
    flex-direction: column;
    height: 100vh;
    background-color: var(--theme-bg-primary);
    color: var(--theme-text-primary);
    font-family: var(--theme-font-family);
    font-size: var(--theme-font-size);
  }

  /* Force form elements to inherit font properties from container */
  :global(.container input),
  :global(.container textarea),
  :global(.container button),
  :global(.container select) {
    font-family: inherit;
    font-size: inherit;
  }
</style>
