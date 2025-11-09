import * as React from "react";
import { Slot } from "@radix-ui/react-slot";
import { cva, type VariantProps } from "class-variance-authority";

import { cn } from "./utils";
import styles from "./button.module.scss";

const buttonVariants = cva(
  styles.button,
  {
    variants: {
      variant: {
        default: styles['button--default'],
        destructive: styles['button--destructive'],
        outline: styles['button--outline'],
        secondary: styles['button--secondary'],
        ghost: styles['button--ghost'],
        link: styles['button--link'],
      },
      size: {
        default: styles['button--default-size'],
        sm: styles['button--sm'],
        lg: styles['button--lg'],
        icon: styles['button--icon'],
      },
    },
    defaultVariants: {
      variant: "default",
      size: "default",
    },
  },
);

const Button = React.forwardRef<
  HTMLButtonElement,
  React.ComponentProps<"button"> &
    VariantProps<typeof buttonVariants> & {
      asChild?: boolean;
    }
>(({ className, variant, size, asChild = false, ...props }, ref) => {
  const Comp = asChild ? Slot : "button";

  return (
    <Comp
      data-slot="button"
      className={cn(buttonVariants({ variant, size, className }))}
      ref={ref}
      {...props}
    />
  );
});

Button.displayName = "Button";

export { Button, buttonVariants };
