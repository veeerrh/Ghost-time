import { writable } from 'svelte/store';

export const currentPage = writable<'timeline' | 'matters' | 'export'>('timeline');
