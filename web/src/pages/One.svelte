<script lang="ts">
	import { onMount, getContext } from "svelte";
	import { lpath } from "../Router.svelte";

	let name: string;
	let disabled = true;

	let pings: number | null = null;

	let pinging = false;

	async function ping() {
		pinging = true;

		const res = await fetch("/api/ping/", { method: "PUT" });
		const json = await res.json();
		pings = json.pings;

		pinging = false;
	}

	onMount(async () => {
		const res = await fetch("/api/ping/");
		if (res.ok) {
			const json = await res.json();
			pings = json.pings;
		} else {
			pings = 0;
		}
	});

	$: disabled = !/^\w+$/.test(name);
</script>

<h1>Page One</h1>

<div>
	{#if pings == null}
		Loading pings...
	{:else}
		Pings: {pings}
		<button disabled={pinging} on:click={ping}>Ping</button>
	{/if}
</div>

<div>
	<label for="name">Name: </label>
	<input id="name" bind:value={name} />
</div>

<button {disabled} on:click={() => $path = `/two/${name}`}>
	Go to two!
</button>
