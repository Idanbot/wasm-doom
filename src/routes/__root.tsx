import { createRootRoute, HeadContent, Outlet, Scripts } from "@tanstack/react-router";
import { asset } from "@/lib/asset";
import appCss from "../styles.css?url";

const APP_NAME = "BLACKSITE";

export const Route = createRootRoute({
  head: () => ({
    meta: [
      { charSet: "utf-8" },
      { name: "viewport", content: "width=device-width, initial-scale=1" },
      { title: APP_NAME },
      { name: "theme-color", content: "#080d11" },
      {
        name: "description",
        content: "BLACKSITE: Project Ifrit. Enter Nadir-7. Break containment. Eliminate the signal.",
      },
    ],
    links: [
      { rel: "icon", type: "image/svg+xml", href: "/favicon.svg" },
      { rel: "stylesheet", href: appCss },
      { rel: "manifest", href: "/manifest.webmanifest" },
      { rel: "preload", href: asset("/hellscan.wasm"), as: "fetch", type: "application/wasm", crossOrigin: "anonymous" },
    ],
  }),
  component: () => (
    <html lang="en" suppressHydrationWarning>
      <head>
        <HeadContent />
      </head>
      <body>
        <Outlet />
        <Scripts />
      </body>
    </html>
  ),
});
