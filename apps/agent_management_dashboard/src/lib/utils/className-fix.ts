/**
 * Utility to safely get className as a string
 * Fixes issues where className might be a DOMTokenList instead of a string
 *
 * @author @darianrosebrook
 */

/**
 * Safely converts className to a string
 * Handles both string and DOMTokenList cases
 */
export function getClassNameAsString(
  element: Element | null | undefined
): string {
  if (!element) {
    return "";
  }

  // If className is already a string, return it
  if (typeof element.className === "string") {
    return element.className;
  }

  // If className is a DOMTokenList (SVG elements), convert to string
  if (element.className && typeof element.className === "object") {
    const classList = element.className as DOMTokenList;
    return Array.from(classList).join(" ");
  }

  // Fallback: try to get attribute
  return element.getAttribute("class") ?? "";
}

/**
 * Polyfill to ensure className.split works correctly
 * This fixes the issue where className.split is not a function
 */
export function polyfillClassNameSplit(): void {
  if (typeof window === "undefined") {
    return;
  }

  // Only patch if not already patched
  if (
    (window as unknown as { __classNameSplitPatched?: boolean })
      .__classNameSplitPatched
  ) {
    return;
  }

  // Patch Element.prototype.className to always return a string
  try {
    const originalClassNameDescriptor = Object.getOwnPropertyDescriptor(
      Element.prototype,
      "className"
    );

    if (originalClassNameDescriptor) {
      Object.defineProperty(Element.prototype, "className", {
        get: function (this: Element) {
          try {
            const getter = originalClassNameDescriptor.get;
            const value =
              getter && typeof getter === "function"
                ? getter.call(this)
                : this.getAttribute("class") ?? "";

            if (typeof value === "string") {
              return value;
            }
            // If it's a DOMTokenList (SVG), convert to string
            if (value && typeof value === "object" && "length" in value) {
              return Array.from(value as DOMTokenList).join(" ");
            }
            return "";
          } catch (error) {
            // Fallback to getAttribute if getter fails
            return this.getAttribute("class") ?? "";
          }
        },
        set: function (this: Element, newValue: string) {
          try {
            const setter = originalClassNameDescriptor.set;
            if (setter && typeof setter === "function") {
              setter.call(this, newValue);
            } else {
              this.setAttribute("class", newValue);
            }
          } catch (error) {
            // Fallback to setAttribute if setter fails
            this.setAttribute("class", newValue);
          }
        },
        configurable: true,
        enumerable: true,
      });

      (
        window as unknown as { __classNameSplitPatched?: boolean }
      ).__classNameSplitPatched = true;
    }
  } catch (error) {
    // Silently fail if polyfill cannot be applied
    console.warn("Failed to apply className polyfill:", error);
  }
}
