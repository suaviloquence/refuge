import { derived, writable, type Writable } from "svelte/store";

export const path: Writable<string> = writable(window.location.pathname);
export const auth: Writable<string> = writable();
export const authGet = derived(auth, (token) => {
  return async (url: string | URL, init: Omit<RequestInit, "headers"> = {}) => {
    return fetch(url, {
      headers: {
        Authorization: `Bearer ${token}`,
      },
      ...init,
    });
  };
});
export const authJson = derived(auth, (token) => {
  return async (
    url: string | URL,
    data: Record<string, any>,
    init: Omit<RequestInit, "body" | "headers"> = {}
  ) => {
    return fetch(url, {
      headers: {
        Authorization: `Bearer ${token}`,
        "Content-Type": "application/json",
      },
      body: JSON.stringify(data),
      ...init,
    });
  };
});
