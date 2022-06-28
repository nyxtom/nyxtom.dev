module.exports = {
    content: [
        "./src/**/*.{html,css,js,jsx,ts,tsx}"
    ],
    theme: {
        container: {
            center: true
        },
        extend: {}
    },
    plugins: [
        require("@tailwindcss/typography")
    ],
}
