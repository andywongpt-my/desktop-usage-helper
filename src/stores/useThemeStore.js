import { create } from "zustand";

/**
 * Theme store — applies dark/light class on <html>.
 * The actual color overrides live in index.css under :root.light selectors.
 */
export const useThemeStore = create((set, get) => ({
  theme: "dark",

  setTheme: (theme) => {
    set({ theme });
    const html = document.documentElement;
    if (theme === "light") {
      html.classList.add("light");
      html.classList.remove("dark");
    } else {
      html.classList.add("dark");
      html.classList.remove("light");
    }
  },

  toggle: () => {
    const next = get().theme === "dark" ? "light" : "dark";
    get().setTheme(next);
    return next;
  },
}));