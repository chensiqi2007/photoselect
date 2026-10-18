/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{vue,js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        darkroom: {
          app: "#121212",
          viewport: "#1E1E1E",
          panel: "#181818",
          border: "#2A2A2A",
          hover: "#262626",
          active: "#E5E5E5",
          text: "#E5E5E5",
          muted: "#8C8C8C",
          disabled: "#4D4D4D",
        },
        label: {
          red: "#EF4444",
          yellow: "#F59E0B",
          green: "#10B981",
          blue: "#3B82F6",
          purple: "#8B5CF6",
        }
      },
      fontFamily: {
        sans: ["Inter", "system-ui", "-apple-system", "sans-serif"],
        mono: ["JetBrains Mono", "Fira Code", "monospace"],
      }
    },
  },
  plugins: [],
}
