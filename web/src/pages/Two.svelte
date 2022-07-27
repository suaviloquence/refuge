<script lang="ts">
	export let name: string;
</script>

<h1>Hi, {name}</h1>
<button on:click={() => history.go(-1)}>Back</button>
