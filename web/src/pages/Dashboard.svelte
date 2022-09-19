<script lang="ts">
  import { onMount } from "svelte";
  import Todo from "../components/Todo.svelte";
  import { auth, authGet, authJson, path } from "../stores";

  let todos: { id: number; text: string; completed: boolean }[] | null = null;

  onMount(async () => {
    $auth = localStorage.getItem("auth");

    if (!auth) $path = "/login";

    const res = await $authGet("/api/todo");

    if (!res.ok) {
      localStorage.removeItem("auth");
      $path = "/login";
    }

    todos = await res.json();
  });

  let disabled = false;

  let text: string;

  const PLACEHOLDERS = [
    "Make a pillow fort",
    "Call family",
    "Contribute to open-source software",
  ];

  async function create() {
    disabled = true;
    const todo = await (
      await $authJson("/api/todo", { text }, { method: "post" })
    ).json();

    todos = [todo, ...todos];
    text = "";

    disabled = false;
  }

  let editMode = false;

  async function clear() {
    todos = await (
      await $authGet("/api/todo/clear", { method: "post" })
    ).json();
  }
</script>

{#if !todos}
  <p>Loading todo list...</p>
{:else}
  <form on:submit|preventDefault={create} {disabled}>
    <div>
      <input
        type="text"
        bind:value={text}
        required
        placeholder={PLACEHOLDERS[
          Math.floor(Math.random() * PLACEHOLDERS.length)
        ]}
      />
      <button type="submit">+</button>
      <button type="button" on:click={() => (editMode = !editMode)}
        >{editMode ? "Done" : "Edit"}</button
      >
      <button type="button" on:click={clear}>Clear done</button>
    </div>
  </form>
  {#each todos as todo, i}
    <Todo {editMode} {...todo} />
  {/each}
{/if}
