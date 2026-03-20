/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      fontFamily: {
        sans: ['Inter', 'sans-serif'],
        mono: ['JetBrains Mono', 'monospace'],
      },
      colors: {
        zinc: {
          50: 'var(--color-theme-50)',
          100: 'var(--color-theme-100)',
          200: 'var(--color-theme-200)',
          300: 'var(--color-theme-300)',
          400: 'var(--color-theme-400)',
          500: 'var(--color-theme-500)',
          600: 'var(--color-theme-600)',
          700: 'var(--color-theme-700)',
          800: 'var(--color-theme-800)',
          900: 'var(--color-theme-900)',
          950: 'var(--color-theme-950)',
        },
        background: 'var(--color-theme-950)',
        surface: 'var(--color-theme-900)',
        border: 'var(--color-theme-800)',
      },
      fontSize: {
        'xs': ['0.875rem', { lineHeight: '1.25rem' }],   // 14px (was 12px)
        'sm': ['1rem', { lineHeight: '1.5rem' }],       // 16px (was 14px)
        'base': ['1.125rem', { lineHeight: '1.75rem' }], // 18px (was 16px)
        'lg': ['1.25rem', { lineHeight: '1.75rem' }],    // 20px (was 18px)
        'xl': ['1.5rem', { lineHeight: '2rem' }],        // 24px (was 20px)
        '2xl': ['1.875rem', { lineHeight: '2.25rem' }],  // 30px (was 24px)
        '3xl': ['2.25rem', { lineHeight: '2.5rem' }],    // 36px (was 30px)
      },
    },
  },
  plugins: [
    require('@tailwindcss/typography'),
  ],
}
