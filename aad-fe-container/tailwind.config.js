/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    "./src/**/*.{html,ts}",
  ],
  theme: {
    extend: {},
  },
  plugins: [
    require('@tailwindcss/forms')({
      strategy: 'class', // avoid resetting Angular Material input / textarea borders
    }),
    require('@tailwindcss/container-queries')
  ],
}

