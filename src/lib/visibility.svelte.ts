// Page visibility is a single global concept (one `document`), so this is a
// module-level singleton rather than something each component tracks
// separately — every timer display-tick effect subscribes to the same
// reactive flag instead of registering its own visibilitychange listener.
class Visibility {
    visible = $state(document.visibilityState === "visible");
}

export const visibility = new Visibility();

document.addEventListener("visibilitychange", () => {
    visibility.visible = document.visibilityState === "visible";
});
