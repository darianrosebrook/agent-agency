"use client";

import * as React from "react";
import { ChevronLeft, ChevronRight } from "lucide-react";
import { DayPicker } from "react-day-picker";

import { cn } from "./utils";
import { buttonVariants } from "./button";
import styles from "./calendar.module.scss";

function Calendar({
  className,
  classNames,
  showOutsideDays = true,
  ...props
}: React.ComponentProps<typeof DayPicker>) {
  return (
    <DayPicker
      showOutsideDays={showOutsideDays}
      className={cn(styles.calendar, className)}
      classNames={{
        months: styles.calendarMonths,
        month: styles.calendarMonth,
        caption: styles.calendarCaption,
        caption_label: styles.calendarCaptionLabel,
        nav: styles.calendarNav,
        nav_button: cn(
          buttonVariants({ variant: "outline" }),
          styles.calendarNavButton,
        ),
        nav_button_previous: styles.calendarNavButtonPrevious,
        nav_button_next: styles.calendarNavButtonNext,
        table: styles.calendarTable,
        head_row: styles.calendarHeadRow,
        head_cell: styles.calendarHeadCell,
        row: styles.calendarRow,
        cell: cn(
          styles.calendarCell,
          props.mode === "range"
            ? styles.calendarCellRange
            : styles.calendarCellSingle,
        ),
        day: cn(buttonVariants({ variant: "ghost" }), styles.calendarDay),
        day_range_start: styles.calendarDayRangeStart,
        day_range_end: styles.calendarDayRangeEnd,
        day_selected: styles.calendarDaySelected,
        day_today: styles.calendarDayToday,
        day_outside: styles.calendarDayOutside,
        day_disabled: styles.calendarDayDisabled,
        day_range_middle: styles.calendarDayRangeMiddle,
        day_hidden: styles.calendarDayHidden,
        ...classNames,
      }}
      components={{
        IconLeft: ({ className, ...props }) => (
          <ChevronLeft className={cn("size-4", className)} {...props} />
        ),
        IconRight: ({ className, ...props }) => (
          <ChevronRight className={cn("size-4", className)} {...props} />
        ),
      }}
      {...props}
    />
  );
}

export { Calendar };
