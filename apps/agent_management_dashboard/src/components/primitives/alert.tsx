import * as React from "react";
import { cva, type VariantProps } from "class-variance-authority";

import { cn } from "./utils";
import styles from "./alert.module.scss";

const alertVariants = cva(styles.alert, {
  variants: {
    variant: {
      default: styles['variant-default'],
      destructive: styles['variant-destructive'],
    },
  },
  defaultVariants: {
    variant: "default",
  },
});

function Alert({
  className,
  variant,
  ...props
}: React.ComponentProps<"div"> & VariantProps<typeof alertVariants>) {
  return (
    <div
      data-slot="alert"
      role="alert"
      className={cn(alertVariants({ variant }), className)}
      {...props}
    />
  );
}

function AlertTitle({ className, ...props }: React.ComponentProps<"div">) {
  return (
    <div
      data-slot="alert-title"
      className={cn(styles.alertTitle, className)}
      {...props}
    />
  );
}

function AlertDescription({
  className,
  ...props
}: React.ComponentProps<"div">) {
  return (
    <div
      data-slot="alert-description"
      className={cn(styles.alertDescription, className)}
      {...props}
    />
  );
}

export { Alert, AlertTitle, AlertDescription };
