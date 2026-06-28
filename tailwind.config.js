/** @type {import('tailwindcss').Config} */
export default {
  content: ["./index.html", "./src/**/*.{js,ts,jsx,tsx}"],
  theme: {
    extend: {
      colors: {
        surface: {
          DEFAULT: "#0f0f0f",
          alt: "#1a1a1a",
          hover: "#252525",
          border: "#2a2a2a",
        },
        accent: {
          DEFAULT: "#6366f1",
          hover: "#5558e6",
          muted: "#4f46e5",
        },
        text: {
          primary: "#e5e5e5",
          secondary: "#a3a3a3",
          muted: "#6b7280",
        },
        danger: {
          DEFAULT: "#ef4444",
          hover: "#dc2626",
        },
        success: {
          DEFAULT: "#22c55e",
        },
        warning: {
          DEFAULT: "#f59e0b",
        },
      },
      fontFamily: {
        sans: [
          "Inter",
          "ui-sans-serif",
          "system-ui",
          "-apple-system",
          "sans-serif",
        ],
        mono: [
          "JetBrains Mono",
          "ui-monospace",
          "SFMono-Regular",
          "monospace",
        ],
      },
    },
  },
  plugins: [],
};
