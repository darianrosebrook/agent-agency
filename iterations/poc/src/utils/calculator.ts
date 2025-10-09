/**
 * Simple calculator utility for mutation testing
 *
 * @author @darianrosebrook
 */

export class Calculator {
  /**
   * Adds two numbers
   */
  add(a: number, b: number): number {
    return a + b;
  }

  /**
   * Subtracts two numbers
   */
  subtract(a: number, b: number): number {
    return a - b;
  }

  /**
   * Multiplies two numbers
   */
  multiply(a: number, b: number): number {
    return a * b;
  }

  /**
   * Divides two numbers
   */
  divide(a: number, b: number): number {
    if (b === 0) {
      throw new Error("Division by zero");
    }
    return a / b;
  }

  /**
   * Checks if a number is even
   */
  isEven(n: number): boolean {
    return n % 2 === 0;
  }

  /**
   * Finds the maximum of two numbers
   */
  max(a: number, b: number): number {
    return a > b ? a : b;
  }

  /**
   * Calculates factorial
   */
  factorial(n: number): number {
    if (n < 0) {
      throw new Error("Factorial of negative number");
    }
    if (n === 0 || n === 1) {
      return 1;
    }
    return n * this.factorial(n - 1);
  }
}
