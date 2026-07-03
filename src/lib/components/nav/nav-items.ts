import { CalendarCheck, ListTodo, BarChart3, Settings } from "@lucide/svelte";
import type { Component } from "svelte";

export type NavItem = { href: string; label: string; icon: Component };

export const NAV: NavItem[] = [
    { href: "/", label: "Today", icon: CalendarCheck },
    { href: "/habits", label: "Habits", icon: ListTodo },
    { href: "/stats", label: "Stats", icon: BarChart3 },
    { href: "/settings", label: "Settings", icon: Settings },
];
