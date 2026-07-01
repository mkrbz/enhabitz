declare const __PLATFORM__: string;

export const platform = __PLATFORM__;
export const isMobile = platform === "android" || platform === "ios";
export const isDesktop = !isMobile;
