<script lang="ts">
  import { onMount } from "svelte";
  import { path } from "../stores";

  let todos: { id: number; text: string; completed: boolean }[] | null = null;

  let auth: string;

  onMount(async () => {
    auth = localStorage.getItem("auth");

    if (!auth) $path = "/login";

    const res = await fetch("/api/todo", {
      headers: {
        authorization: `Bearer ${auth}`,
      },
    });

    if (!res.ok) {
      localStorage.removeItem("auth");
      $path = "/login";
    }

    todos = await res.json();
  });

  let disabled = false;

  async function toggle(i: number) {
    disabled = true;

    const todo = todos[i];
    todo.completed = !todo.completed;

    await fetch(`/api/todo/${todo.id}`, {
      method: "put",
      headers: {
        authorization: `Bearer ${auth}`,
        "Content-Type": "application/json",
      },
      body: JSON.stringify(todo),
    });

    disabled = false;
  }

  let text: string;

  const PLACEHOLDERS = [
    "Make a pillow fort",
    "Call family",
    "Contribute to open-source software",
  ];

  async function create() {
    disabled = true;
    const todo = await (
      await fetch("/api/todo", {
        method: "post",
        headers: {
          Authorization: `Bearer ${auth}`,
          "Content-Type": "application/json",
        },
        body: JSON.stringify({ text }),
      })
    ).json();

    todos = [todo, ...todos];
    text = "";

    disabled = false;
  }
</script>

{#if !todos}
  <p>Loading todo list...</p>
{:else}
  <form on:submit|preventDefault={create} {disabled}>
    <input
      type="text"
      bind:value={text}
      required
      placeholder={PLACEHOLDERS[
        Math.floor(Math.random() * PLACEHOLDERS.length)
      ]}
    />
    <button type="submit">+</button>
  </form>
  {#each todos as todo, i}
    <div>
      <input
        id={todo.id.toString()}
        type="checkbox"
        checked={todo.completed}
        on:click={() => toggle(i)}
        {disabled}
      />
      <label for={todo.id.toString()}>{todo.text}</label>
    </div>
  {/each}
{/if}
