/** @type {import('tailwindcss').Config} */
export default {
  content: ["./index.html", "./src/**/*.{js,jsx,ts,tsx}"],
  darkMode: "class",
  theme: {
    extend: {
      colors: {
        // Dark creative-editor palette (mirror of Pipeline Photo's gray scale)
        "gray-950": "#0a0a0c",
        "gray-900": "#111114",
        "gray-850": "#18181c",
        "gray-800": "#1e1e24",
        "gray-750": "#25252c",
        "gray-700": "#2d2d35",
        "gray-600": "#3a3a44",
        "gray-500": "#4b4b57",
        accent: "#5e8cff",
        ok: "#4ade80",
        warn: "#fbbf24",
        danger: "#f87171",
      },
      fontFamily: {
        sans: [
          "-apple-system",
          "BlinkMacSystemFont",
          "Segoe UI Variable",
          "Segoe UI",
          "Roboto",
          "Helvetica Neue",
          "sans-serif",
        ],
        mono: [
          "JetBrains Mono",
          "Cascadia Code",
          "Consolas",
          "SF Mono",
          "Menlo",
          "monospace",
        ],
      },
      animation: {
        "pulse-slow": "pulse 3s cubic-bezier(0.4, 0, 0.6, 1) infinite",
        "fade-in": "fadeIn 0.3s ease-out",
      },
      keyframes: {
        fadeIn: {
          from: { opacity: 0, transform: "translateY(4px)" },
          to: { opacity: 1, transform: "translateY(0)" },
        },
      },
    },
  },
  plugins: [],
};
