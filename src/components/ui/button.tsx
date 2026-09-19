import * as React from "react";
import { Slot } from "@radix-ui/react-slot";
import { cva, type VariantProps } from "class-variance-authority";
import { cn } from "@/lib/utils";

const buttonVariants = cva(
  "inline-flex items-center justify-center gap-2 whitespace-nowrap font-display uppercase tracking-[0.16em] transition-opacity duration-[var(--motion-quick)] ease-[var(--ease-out)] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-danger/70 disabled:pointer-events-none disabled:opacity-40 border-2",
  {
    variants: {
      variant: {
        primary:
          "border-danger/70 bg-danger text-fg shadow-[inset_0_1px_0_color-mix(in_oklab,var(--color-fg)_30%,transparent)] hover:opacity-90 active:scale-[0.98]",
        ghost:
          "border-border bg-elevated text-fg hover:border-steel/40 hover:bg-surface",
        danger: "border-danger bg-danger text-fg hover:opacity-90",
      },
      size: {
        md: "h-11 px-5 text-sm",
        lg: "h-12 px-6 text-base",
        sm: "h-9 px-3 text-xs",
      },
    },
    defaultVariants: { variant: "primary", size: "md" },
  },
);

export function Button({
  className,
  variant,
  size,
  asChild = false,
  ...props
}: React.ComponentProps<"button"> &
  VariantProps<typeof buttonVariants> & { asChild?: boolean }) {
  const Comp = asChild ? Slot : "button";
  return (
    <Comp className={cn(buttonVariants({ variant, size }), className)} {...props} />
  );
}
