<script lang="ts">
  import { path } from "../stores";

  export let mode: "login" | "signup" = "login";

  let username: string;
  let password: string;

  let error: string | null = null;

  let disabled = false;
  async function submit() {
    disabled = true;

    const res = await fetch(`/api/user/${mode}`, {
      method: "post",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({ username, password }),
    });

    if (res.ok) {
      const auth = await res.text();
      localStorage.setItem("auth", auth);
      $path = "/";
    } else {
      error = await res.text();
    }

    disabled = false;
  }
</script>

<form on:submit|preventDefault={submit}>
  <h1>{mode === "login" ? "Log In" : "Sign Up"}</h1>

  <div>
    <label for="username">Username: </label>
    <input type="text" id="username" required bind:value={username} />
  </div>
  <div>
    <label for="password">Password: </label>
    <input type="password" id="password" required bind:value={password} />
  </div>

  {#if error}
    <div>{error}</div>
  {/if}

  <button type="submit" {disabled}>{mode}</button>
  <button
    on:click={() => (mode = mode === "login" ? "signup" : "login")}
    type="button">Switch</button
  >
</form>
