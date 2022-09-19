<script lang="ts" context="module">
  export interface Todo {
    id: number;
    text: string;
    completed: boolean;
    cleared: boolean;
  }
</script>

<script lang="ts">
  import { authJson } from "../stores";

  export let id: number;
  export let text: string;
  export let completed: boolean;

  export let editMode: boolean;

  let disabled = false;

  let stored = text;

  async function update() {
    disabled = true;

    const res = await (
      await $authJson(`/api/todo/${id}`, { text, completed }, { method: "put" })
    ).json();

    id = res.id;
    text = res.text;
    completed = res.completed;
    stored = res.text;

    disabled = false;
  }

  async function toggle() {
    completed = !completed;

    return update();
  }

  async function checkUpdate(e: boolean) {
    if (!e) {
      if (stored !== text) return update();
    }
  }

  $: checkUpdate(editMode);

  let focused = false;
</script>

<div>
  <input
    id={id.toString()}
    type="checkbox"
    checked={completed}
    on:click={toggle}
    {disabled}
  />
  {#if editMode}
    <form on:submit|preventDefault={update}>
      <input
        type="text"
        bind:value={text}
        required
        on:blur={() => (focused = false)}
        on:focus={() => (focused = true)}
      />
      <button class:focused type="button" on:click={() => (text = stored)}
        >x</button
      >
    </form>
  {:else}
    <label for={id.toString()}>{text}</label>
  {/if}
</div>

<style>
  button {
    display: none;
  }
  button.focused {
    display: inline;
  }

  form {
    display: inline;
  }
</style>
