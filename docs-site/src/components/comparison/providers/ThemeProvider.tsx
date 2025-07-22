/**
 * Theme Provider for FHIRPath Comparison Page
 * Provides dark/light mode support with Mantine integration
 */

import { createContext, useContext, useEffect, useState } from "react";
import { MantineProvider, createTheme } from "@mantine/core";
import "@mantine/core/styles.css";
import "@mantine/charts/styles.css";

// Linear Design System tokens
const linearColors = {
	light: {
		surface: "#fff",
		surfaceSecondary: "#fafbfc",
		border: "#e3e8ef",
		borderLight: "#f4f6fa",
		text: "#1a1d23",
		textSecondary: "#6b7585",
		textMuted: "#a1a1aa",
		primary: "#7b61ff",
		accent: "#ffb020",
		error: "#ff647c",
		warning: "#ffb020",
		success: "#2ecc40",
	},
	dark: {
		surface: "#18181b",
		surfaceSecondary: "#232326",
		border: "#232326",
		borderLight: "#232326",
		text: "#fff",
		textSecondary: "#a1a1aa",
		textMuted: "#71717a",
		primary: "#b69eff",
		accent: "#ffd580",
		error: "#ff647c",
		warning: "#ffd580",
		success: "#2ecc40",
	},
};

const linearRadii = { xs: "4px", sm: "6px", md: "8px", lg: "12px", xl: "16px" };
const linearSpacing = {
	xs: "4px",
	sm: "8px",
	md: "12px",
	lg: "16px",
	xl: "24px",
};
const linearFontSizes = {
	xs: "14px",
	sm: "15px",
	md: "16px",
	lg: "18px",
	xl: "20px",
};

// Ensure all color arrays are 10-element arrays for Mantine
const gray = [
	"#fafafa",
	"#f4f4f5",
	"#e4e4e7",
	"#d4d4d8",
	"#a1a1aa",
	"#71717a",
	"#52525b",
	"#3f3f46",
	"#27272a",
	"#18181b",
] as const;
const success = [
	"#f0fdf4",
	"#dcfce7",
	"#bbf7d0",
	"#86efac",
	"#4ade80",
	"#22c55e",
	"#16a34a",
	"#15803d",
	"#166534",
	"#14532d",
] as const;
const warning = [
	"#fffbeb",
	"#fef3c7",
	"#fde68a",
	"#fcd34d",
	"#fbbf24",
	"#f59e0b",
	"#d97706",
	"#b45309",
	"#92400e",
	"#78350f",
] as const;
const error = [
	"#fef2f2",
	"#fecaca",
	"#fca5a5",
	"#f87171",
	"#ef4444",
	"#dc2626",
	"#b91c1c",
	"#991b1b",
	"#7f1d1d",
	"#450a0a",
] as const;

// Define custom color arrays for Mantine (10 shades each)
const surface = [
	"#fff",
	"#fff",
	"#fff",
	"#fff",
	"#fff",
	"#fff",
	"#fff",
	"#fff",
	"#fff",
	"#fff",
] as const;
const surfaceDark = [
	"#18181b",
	"#18181b",
	"#18181b",
	"#18181b",
	"#18181b",
	"#18181b",
	"#18181b",
	"#18181b",
	"#18181b",
	"#18181b",
] as const;
const surfaceSecondary = [
	"#fafbfc",
	"#fafbfc",
	"#fafbfc",
	"#fafbfc",
	"#fafbfc",
	"#fafbfc",
	"#fafbfc",
	"#fafbfc",
	"#fafbfc",
	"#fafbfc",
] as const;
const surfaceSecondaryDark = [
	"#232326",
	"#232326",
	"#232326",
	"#232326",
	"#232326",
	"#232326",
	"#232326",
	"#232326",
	"#232326",
	"#232326",
] as const;
const border = [
	"#e3e8ef",
	"#e3e8ef",
	"#e3e8ef",
	"#e3e8ef",
	"#e3e8ef",
	"#e3e8ef",
	"#e3e8ef",
	"#e3e8ef",
	"#e3e8ef",
	"#e3e8ef",
] as const;
const borderDark = [
	"#232326",
	"#232326",
	"#232326",
	"#232326",
	"#232326",
	"#232326",
	"#232326",
	"#232326",
	"#232326",
	"#232326",
] as const;
const borderLight = [
	"#f4f6fa",
	"#f4f6fa",
	"#f4f6fa",
	"#f4f6fa",
	"#f4f6fa",
	"#f4f6fa",
	"#f4f6fa",
	"#f4f6fa",
	"#f4f6fa",
	"#f4f6fa",
] as const;
const borderLightDark = [
	"#232326",
	"#232326",
	"#232326",
	"#232326",
	"#232326",
	"#232326",
	"#232326",
	"#232326",
	"#232326",
	"#232326",
] as const;
const text = [
	"#1a1d23",
	"#1a1d23",
	"#1a1d23",
	"#1a1d23",
	"#1a1d23",
	"#1a1d23",
	"#1a1d23",
	"#1a1d23",
	"#1a1d23",
	"#1a1d23",
] as const;
const textDark = [
	"#fff",
	"#fff",
	"#fff",
	"#fff",
	"#fff",
	"#fff",
	"#fff",
	"#fff",
	"#fff",
	"#fff",
] as const;
const textSecondary = [
	"#6b7585",
	"#6b7585",
	"#6b7585",
	"#6b7585",
	"#6b7585",
	"#6b7585",
	"#6b7585",
	"#6b7585",
	"#6b7585",
	"#6b7585",
] as const;
const textSecondaryDark = [
	"#a1a1aa",
	"#a1a1aa",
	"#a1a1aa",
	"#a1a1aa",
	"#a1a1aa",
	"#a1a1aa",
	"#a1a1aa",
	"#a1a1aa",
	"#a1a1aa",
	"#a1a1aa",
] as const;
const textMuted = [
	"#a1a1aa",
	"#a1a1aa",
	"#a1a1aa",
	"#a1a1aa",
	"#a1a1aa",
	"#a1a1aa",
	"#a1a1aa",
	"#a1a1aa",
	"#a1a1aa",
	"#a1a1aa",
] as const;
const textMutedDark = [
	"#71717a",
	"#71717a",
	"#71717a",
	"#71717a",
	"#71717a",
	"#71717a",
	"#71717a",
	"#71717a",
	"#71717a",
	"#71717a",
] as const;
const primary = [
	"#7b61ff",
	"#7b61ff",
	"#7b61ff",
	"#7b61ff",
	"#7b61ff",
	"#7b61ff",
	"#7b61ff",
	"#7b61ff",
	"#7b61ff",
	"#7b61ff",
] as const;
const primaryDark = [
	"#b69eff",
	"#b69eff",
	"#b69eff",
	"#b69eff",
	"#b69eff",
	"#b69eff",
	"#b69eff",
	"#b69eff",
	"#b69eff",
	"#b69eff",
] as const;
const accent = [
	"#ffb020",
	"#ffb020",
	"#ffb020",
	"#ffb020",
	"#ffb020",
	"#ffb020",
	"#ffb020",
	"#ffb020",
	"#ffb020",
	"#ffb020",
] as const;
const accentDark = [
	"#ffd580",
	"#ffd580",
	"#ffd580",
	"#ffd580",
	"#ffd580",
	"#ffd580",
	"#ffd580",
	"#ffd580",
	"#ffd580",
	"#ffd580",
] as const;
const errorColor = [
	"#ff647c",
	"#ff647c",
	"#ff647c",
	"#ff647c",
	"#ff647c",
	"#ff647c",
	"#ff647c",
	"#ff647c",
	"#ff647c",
	"#ff647c",
] as const;
const warningColor = [
	"#ffb020",
	"#ffb020",
	"#ffb020",
	"#ffb020",
	"#ffb020",
	"#ffb020",
	"#ffb020",
	"#ffb020",
	"#ffb020",
	"#ffb020",
] as const;
const warningColorDark = [
	"#ffd580",
	"#ffd580",
	"#ffd580",
	"#ffd580",
	"#ffd580",
	"#ffd580",
	"#ffd580",
	"#ffd580",
	"#ffd580",
	"#ffd580",
] as const;
const successColor = [
	"#2ecc40",
	"#2ecc40",
	"#2ecc40",
	"#2ecc40",
	"#2ecc40",
	"#2ecc40",
	"#2ecc40",
	"#2ecc40",
	"#2ecc40",
	"#2ecc40",
] as const;

// Language and semantic colors for charts (preserved)
const languageColors = {
	rust: [
		"#fef7f0",
		"#fde8d7",
		"#fbd0a8",
		"#f7b179",
		"#f3924a",
		"#ef7b2c",
		"#d4621a",
		"#b8520f",
		"#9c4208",
		"#803304",
	] as const,
	javascript: [
		"#fffbeb",
		"#fef3c7",
		"#fde68a",
		"#fcd34d",
		"#fbbf24",
		"#f59e0b",
		"#d97706",
		"#b45309",
		"#92400e",
		"#78350f",
	] as const,
	python: [
		"#f0fdf4",
		"#dcfce7",
		"#bbf7d0",
		"#86efac",
		"#4ade80",
		"#22c55e",
		"#16a34a",
		"#15803d",
		"#166534",
		"#14532d",
	] as const,
	java: [
		"#fff7ed",
		"#ffedd5",
		"#fed7aa",
		"#fdba74",
		"#fb923c",
		"#f97316",
		"#ea580c",
		"#dc2626",
		"#b91c1c",
		"#991b1b",
	] as const,
	csharp: [
		"#faf5ff",
		"#f3e8ff",
		"#e9d5ff",
		"#d8b4fe",
		"#c084fc",
		"#a855f7",
		"#9333ea",
		"#7c3aed",
		"#6d28d9",
		"#5b21b6",
	] as const,
	go: [
		"#f0fdfa",
		"#ccfbf1",
		"#99f6e4",
		"#5eead4",
		"#2dd4bf",
		"#14b8a6",
		"#0d9488",
		"#0f766e",
		"#115e59",
		"#134e4a",
	] as const,
	clojure: [
		"#eff6ff",
		"#dbeafe",
		"#bfdbfe",
		"#93c5fd",
		"#60a5fa",
		"#3b82f6",
		"#2563eb",
		"#1d4ed8",
		"#1e40af",
		"#1e3a8a",
	] as const,
	gray,
	success,
	warning,
	error,
};

export const chartColors = {
	light: {
		rust: "#ef7b2c",
		javascript: "#f59e0b",
		python: "#22c55e",
		java: "#f97316",
		csharp: "#a855f7",
		go: "#14b8a6",
		clojure: "#3b82f6",
		success: "#22c55e",
		warning: "#f59e0b",
		error: "#dc2626",
		grid: linearColors.light.border,
		text: linearColors.light.text,
		textSecondary: linearColors.light.textSecondary,
		textMuted: linearColors.light.textMuted,
		background: linearColors.light.surface,
		surface: linearColors.light.surface,
		surfaceSecondary: linearColors.light.surfaceSecondary,
		border: linearColors.light.border,
		borderLight: linearColors.light.borderLight,
	},
	dark: {
		rust: "#f7b179",
		javascript: "#fbbf24",
		python: "#4ade80",
		java: "#fb923c",
		csharp: "#c084fc",
		go: "#2dd4bf",
		clojure: "#60a5fa",
		success: "#4ade80",
		warning: "#fbbf24",
		error: "#f87171",
		grid: linearColors.dark.border,
		text: linearColors.dark.text,
		textSecondary: linearColors.dark.textSecondary,
		textMuted: linearColors.dark.textMuted,
		background: linearColors.dark.surface,
		surface: linearColors.dark.surface,
		surfaceSecondary: linearColors.dark.surfaceSecondary,
		border: linearColors.dark.border,
		borderLight: linearColors.dark.borderLight,
	},
};

// Theme context
type ColorScheme = "light" | "dark";

interface ThemeContextType {
	colorScheme: ColorScheme;
	toggleColorScheme: () => void;
	chartColors: typeof chartColors.light;
}

const ThemeContext = createContext<ThemeContextType | undefined>(undefined);

// Hook to use theme context
export const useComparisonTheme = () => {
	const context = useContext(ThemeContext);
	if (!context) {
		throw new Error(
			"useComparisonTheme must be used within a ComparisonThemeProvider",
		);
	}
	return context;
};

// Theme provider component
interface ComparisonThemeProviderProps {
	children: React.ReactNode;
}

export function ComparisonThemeProvider({
	children,
}: ComparisonThemeProviderProps) {
	const [colorScheme, setColorScheme] = useState<ColorScheme>("light");

	// Initialize theme from localStorage or system preference
	useEffect(() => {
		const savedTheme = localStorage.getItem("comparison-theme") as ColorScheme;
		if (savedTheme && (savedTheme === "light" || savedTheme === "dark")) {
			setColorScheme(savedTheme);
		} else {
			// Detect system preference
			const prefersDark = window.matchMedia(
				"(prefers-color-scheme: dark)",
			).matches;
			setColorScheme(prefersDark ? "dark" : "light");
		}
	}, []);

	// Toggle theme function
	const toggleColorScheme = () => {
		const newScheme = colorScheme === "dark" ? "light" : "dark";
		setColorScheme(newScheme);
		localStorage.setItem("comparison-theme", newScheme);
	};

	// Mantine theme using Linear tokens
	const theme = createTheme({
		colors: {
			...languageColors,
			surface,
			surfaceDark,
			surfaceSecondary,
			surfaceSecondaryDark,
			border,
			borderDark,
			borderLight,
			borderLightDark,
			text,
			textDark,
			textSecondary,
			textSecondaryDark,
			textMuted,
			textMutedDark,
			primary,
			primaryDark,
			accent,
			accentDark,
			error: errorColor,
			warning: warningColor,
			warningDark: warningColorDark,
			success: successColor,
		},
		primaryColor: "rust", // or "primary" if you want to use your custom color as the main accent
		fontFamily:
			'Inter, -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif',
		fontFamilyMonospace:
			'"JetBrains Mono", "SF Mono", Monaco, Inconsolata, "Roboto Mono", Consolas, "Courier New", monospace',
		fontSizes: linearFontSizes,
		radius: linearRadii,
		spacing: linearSpacing,
		headings: {
			fontFamily:
				'Inter, -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif',
			fontWeight: "600",
			sizes: {
				h1: { fontSize: "2.25rem", lineHeight: "2.5rem" },
				h2: { fontSize: "1.875rem", lineHeight: "2.25rem" },
				h3: { fontSize: "1.5rem", lineHeight: "2rem" },
				h4: { fontSize: "1.25rem", lineHeight: "1.75rem" },
				h5: { fontSize: "1.125rem", lineHeight: "1.75rem" },
				h6: { fontSize: "1rem", lineHeight: "1.5rem" },
			},
		},
		shadows: {
			xs: colorScheme === 'dark'
				? "0 1px 2px 0 rgba(0, 0, 0, 0.3)"
				: "0 1px 2px 0 rgba(0, 0, 0, 0.05)",
			sm: colorScheme === 'dark'
				? "0 1px 3px 0 rgba(0, 0, 0, 0.4), 0 1px 2px -1px rgba(0, 0, 0, 0.4)"
				: "0 1px 3px 0 rgba(0, 0, 0, 0.1), 0 1px 2px -1px rgba(0, 0, 0, 0.1)",
			md: colorScheme === 'dark'
				? "0 4px 6px -1px rgba(0, 0, 0, 0.4), 0 2px 4px -2px rgba(0, 0, 0, 0.4)"
				: "0 4px 6px -1px rgba(0, 0, 0, 0.1), 0 2px 4px -2px rgba(0, 0, 0, 0.1)",
			lg: colorScheme === 'dark'
				? "0 10px 15px -3px rgba(0, 0, 0, 0.5), 0 4px 6px -4px rgba(0, 0, 0, 0.5)"
				: "0 10px 15px -3px rgba(0, 0, 0, 0.1), 0 4px 6px -4px rgba(0, 0, 0, 0.1)",
			xl: colorScheme === 'dark'
				? "0 20px 25px -5px rgba(0, 0, 0, 0.6), 0 8px 10px -6px rgba(0, 0, 0, 0.6)"
				: "0 20px 25px -5px rgba(0, 0, 0, 0.1), 0 8px 10px -6px rgba(0, 0, 0, 0.1)",
		},
		components: {
			Container: {
				defaultProps: {
					size: "xl",
				},
			},
			Card: {
				styles: {
					root: {
						transition: "all 0.2s ease",
						background: linearColors[colorScheme].surface,
						color: linearColors[colorScheme].text,
						borderColor: linearColors[colorScheme].border,
						"&:hover": {
							transform: "translateY(-1px)",
							boxShadow: "0 4px 24px 0 rgba(0,0,0,0.04)",
						},
					},
				},
			},
			Tabs: {
				styles: {
					tab: {
						fontWeight: 500,
						transition: "all 0.15s ease",
						color: linearColors[colorScheme].textSecondary,
						"&[data-active]": {
							color: linearColors[colorScheme].primary,
							borderColor: linearColors[colorScheme].primary,
						},
					},
				},
			},
		},
	});

	// Get current chart colors
	const currentChartColors = chartColors[colorScheme];

	const themeContextValue: ThemeContextType = {
		colorScheme,
		toggleColorScheme,
		chartColors: currentChartColors,
	};

	return (
		<ThemeContext.Provider value={themeContextValue}>
			<MantineProvider theme={theme} defaultColorScheme={colorScheme}>
				{children}
			</MantineProvider>
		</ThemeContext.Provider>
	);
}

// Theme toggle button component
import { ActionIcon } from "@mantine/core";
import { IconSun, IconMoon } from "@tabler/icons-react";

export function ThemeToggle() {
	const { colorScheme, toggleColorScheme } = useComparisonTheme();

	return (
		<ActionIcon
			onClick={toggleColorScheme}
			size="lg"
			variant="subtle"
			color={colorScheme === "dark" ? "yellow" : "blue"}
			title={`Switch to ${colorScheme === "dark" ? "light" : "dark"} mode`}
			aria-label={`Switch to ${colorScheme === "dark" ? "light" : "dark"} mode`}
		>
			{colorScheme === "dark" ? (
				<IconSun size="1.2rem" />
			) : (
				<IconMoon size="1.2rem" />
			)}
		</ActionIcon>
	);
}
