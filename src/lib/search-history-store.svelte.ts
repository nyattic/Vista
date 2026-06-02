const KEY = 'vista.searchHistory';
const MAX = 20;

function load(): string[] {
  try {
    const raw = localStorage.getItem(KEY);
    return raw ? (JSON.parse(raw) as string[]) : [];
  } catch {
    return [];
  }
}

class SearchHistoryStore {
  items = $state<string[]>(load());

  private persist() {
    localStorage.setItem(KEY, JSON.stringify(this.items));
  }

  add(query: string) {
    const q = query.trim();
    if (!q) return;
    this.items = [q, ...this.items.filter((x) => x !== q)].slice(0, MAX);
    this.persist();
  }

  remove(query: string) {
    this.items = this.items.filter((x) => x !== query);
    this.persist();
  }

  clear() {
    this.items = [];
    this.persist();
  }
}

export const searchHistoryStore = new SearchHistoryStore();
