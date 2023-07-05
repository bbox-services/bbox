/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    './src/*.css',
    '../../bbox-core/templates/*.html',
    '../../bbox-feature-server/templates/*.html',
    '../templates/*.html'
  ],
  theme: {
    extend: {},
  },
  plugins: [require("@tailwindcss/typography"), require("daisyui")],
  daisyui: {
    themes: ["cupcake"],
  },
}
