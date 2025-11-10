import * as React from "react";

import { cn } from "./utils";
import styles from "./textarea.module.scss";

function Textarea({ className, ...props }: React.ComponentProps<"textarea">) {
  return (
    <textarea
      data-slot="textarea"
      className={cn(styles.textarea, className)}
      {...props}
    />
  );
}

export { Textarea };
