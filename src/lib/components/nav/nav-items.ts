import { CalendarCheck, ListTodo, Settings } from "@lucide/svelte";
import type { Component } from "svelte";

export type NavItem = { href: string; label: string; icon: Component };

export const NAV: NavItem[] = [
    { href: "/", label: "Today", icon: CalendarCheck },
    { href: "/habits", label: "Habits", icon: ListTodo },
    { href: "/settings", label: "Settings", icon: Settings },
];
