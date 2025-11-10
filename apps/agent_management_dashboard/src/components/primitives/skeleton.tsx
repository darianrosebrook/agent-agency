import * as React from "react";
import { cn } from "./utils";
import styles from "./skeleton.module.scss";

function Skeleton({ className, ...props }: React.ComponentProps<"div">) {
  return (
    <div
      data-slot="skeleton"
      className={cn(styles.skeleton, className)}
      {...props}
    />
  );
}

export { Skeleton };
