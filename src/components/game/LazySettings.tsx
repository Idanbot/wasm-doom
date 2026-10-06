import { lazy, Suspense } from "react";
import type { SettingsProps } from "./Settings";

/**
 * Settings is the only consumer of `@radix-ui/react-dialog`, so it is kept out
 * of the boot chunk. `Menu` and `Pause` share one lazy instance (and therefore
 * one Suspense boundary) rather than each declaring their own.
 */
const Settings = lazy(() =>
  import("./Settings").then((m) => ({ default: m.Settings })),
);

export function LazySettings(props: SettingsProps) {
  return (
    <Suspense fallback={null}>
      <Settings {...props} />
    </Suspense>
  );
}

export type { SettingsProps };