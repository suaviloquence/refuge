import { writable, type Writable } from "svelte/store";

export const path: Writable<string> = writable(window.location.pathname);
