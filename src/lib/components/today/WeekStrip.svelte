<script lang="ts">
    import { ChevronLeft, ChevronRight } from "@lucide/svelte";

    const DAY_LETTERS = ["S", "M", "T", "W", "T", "F", "S"];
    const MONTHS = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun",
        "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    const SWIPE_THRESHOLD = 40;

    let {
        selected = $bindable(),
        activityFor,
    }: {
        selected: Date;
        activityFor: (date: Date) => "full" | "partial" | "none";
    } = $props();

    function localDateKey(date: Date): string {
        const y = date.getFullYear();
        const m = String(date.getMonth() + 1).padStart(2, "0");
        const d = String(date.getDate()).padStart(2, "0");
        return `${y}-${m}-${d}`;
    }

    function addDays(date: Date, n: number): Date {
        const d = new Date(date);
        d.setDate(d.getDate() + n);
        return d;
    }

    function mondayOf(date: Date): Date {
        return addDays(date, -((date.getDay() + 6) % 7));
    }

    const today = new Date();
    const todayKey = localDateKey(today);

    // The visible 7-day window — independent of `selected` after mount, so
    // browsing weeks doesn't require reloading anything (all data is already
    // in memory; a week is just 7 date computations).
    let weekStart = $state(mondayOf(selected));

    const days = $derived(
        Array.from({ length: 7 }, (_, i) => addDays(weekStart, i)),
    );

    const monthLabel = $derived.by(() => {
        const mid = days[3];
        return `${MONTHS[mid.getMonth()]} ${mid.getFullYear()}`;
    });

    function prevWeek() {
        weekStart = addDays(weekStart, -7);
    }

    function nextWeek() {
        weekStart = addDays(weekStart, 7);
    }

    let touchStartX = 0;

    function onTouchStart(e: TouchEvent) {
        touchStartX = e.touches[0].clientX;
    }

    function onTouchEnd(e: TouchEvent) {
        const dx = e.changedTouches[0].clientX - touchStartX;
        if (dx > SWIPE_THRESHOLD) prevWeek();
        else if (dx < -SWIPE_THRESHOLD) nextWeek();
    }
</script>

<div class="flex flex-col gap-1.5 px-1">
    <div class="flex items-center justify-between">
        <button
            onclick={prevWeek}
            class="p-1 text-muted-foreground hover:text-foreground"
            aria-label="Previous week"
        >
            <ChevronLeft class="h-4 w-4" />
        </button>
        <span class="text-xs font-medium text-muted-foreground">{monthLabel}</span>
        <button
            onclick={nextWeek}
            class="p-1 text-muted-foreground hover:text-foreground"
            aria-label="Next week"
        >
            <ChevronRight class="h-4 w-4" />
        </button>
    </div>

    <div
        class="flex justify-between gap-1"
        role="group"
        ontouchstart={onTouchStart}
        ontouchend={onTouchEnd}
    >
        {#each days as date (date.getTime())}
            {@const key = localDateKey(date)}
            {@const isToday = key === todayKey}
            {@const isSelected = key === localDateKey(selected)}
            {@const activity = activityFor(date)}
            <button
                onclick={() => (selected = date)}
                class="flex flex-1 flex-col items-center gap-1 rounded-lg py-1 transition-colors"
            >
                <span class="text-xs text-muted-foreground">{DAY_LETTERS[date.getDay()]}</span>
                <div
                    class={`flex h-9 w-9 items-center justify-center rounded-full text-sm tabular-nums transition-colors ${
                        isSelected
                            ? "bg-primary text-primary-foreground font-semibold"
                            : isToday
                              ? "ring-2 ring-primary/60 text-foreground"
                              : "text-foreground"
                    }`}
                >
                    {date.getDate()}
                </div>
                <div
                    class={`h-1.5 w-1.5 rounded-full ${
                        activity === "full"
                            ? "bg-primary"
                            : activity === "partial"
                              ? "bg-primary/40"
                              : "bg-transparent"
                    }`}
                ></div>
            </button>
        {/each}
    </div>
</div>
