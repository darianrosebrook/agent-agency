/**
 * FormField - Compound component combining Input with Label and validation
 * 
 * @author @darianrosebrook
 * 
 * Form field optimized for dashboard forms.
 * Includes label, input, helper text, and error states.
 */

// import React from "react";
import { Input, type InputProps } from "../../primitives/Input";
import { Text } from "../../primitives/Text";

export interface FormFieldProps extends Omit<InputProps, "id"> {
  /** Field label */
  label: string;
  /** Field ID (for label association) */
  id: string;
  /** Helper text or error message */
  helperText?: string;
  /** Whether field has error */
  error?: boolean;
  /** Whether field is required */
  required?: boolean;
  /** Additional CSS class for wrapper */
  wrapperClassName?: string;
}

export function FormField({
  label,
  id,
  helperText,
  error = false,
  required = false,
  wrapperClassName,
  className,
  ...inputProps
}: FormFieldProps) {
  return (
    <div
      className={wrapperClassName}
      style={{
        display: "flex",
        flexDirection: "column",
        gap: "var(--spacing-2)",
        contain: "layout style",
      }}
    >
      {/* Label */}
      <label
        htmlFor={id}
        style={{
          fontSize: "0.875rem",
          fontWeight: "var(--font-weight-medium)",
          color: error ? "var(--color-error)" : "var(--color-text-primary)",
          fontFamily: "var(--font-family-display)",
        }}
      >
        {label}
        {required && (
          <span style={{ color: "var(--color-error)", marginLeft: "4px" }}>
            *
          </span>
        )}
      </label>

      {/* Input Field */}
      <Input
        {...inputProps}
        id={id}
        required={required}
        error={error}
        className={className || ""}
      />

      {/* Helper Text or Error */}
      {helperText && (
        <Text
          variant="paragraph-small"
          color={error ? "error" : "muted"}
          style={{ fontSize: "0.75rem" }}
        >
          {helperText}
        </Text>
      )}
    </div>
  );
}

export default FormField;


