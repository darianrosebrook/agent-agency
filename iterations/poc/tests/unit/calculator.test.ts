/**
 * Tests for Calculator utility
 *
 * @author @darianrosebrook
 */

import { describe, expect, it } from "@jest/globals";
import { Calculator } from "../../src/utils/calculator";

describe("Calculator", () => {
  let calculator: Calculator;

  beforeEach(() => {
    calculator = new Calculator();
  });

  describe("add", () => {
    it("should add two positive numbers", () => {
      expect(calculator.add(2, 3)).toBe(5);
    });

    it("should add positive and negative numbers", () => {
      expect(calculator.add(5, -3)).toBe(2);
    });

    it("should add two negative numbers", () => {
      expect(calculator.add(-2, -3)).toBe(-5);
    });

    it("should add zero", () => {
      expect(calculator.add(0, 5)).toBe(5);
      expect(calculator.add(5, 0)).toBe(5);
    });
  });

  describe("subtract", () => {
    it("should subtract two positive numbers", () => {
      expect(calculator.subtract(5, 3)).toBe(2);
    });

    it("should subtract with negative result", () => {
      expect(calculator.subtract(3, 5)).toBe(-2);
    });

    it("should subtract negative numbers", () => {
      expect(calculator.subtract(-5, -3)).toBe(-2);
    });

    it("should subtract zero", () => {
      expect(calculator.subtract(5, 0)).toBe(5);
    });
  });

  describe("multiply", () => {
    it("should multiply two positive numbers", () => {
      expect(calculator.multiply(3, 4)).toBe(12);
    });

    it("should multiply with negative numbers", () => {
      expect(calculator.multiply(3, -4)).toBe(-12);
      expect(calculator.multiply(-3, -4)).toBe(12);
    });

    it("should multiply by zero", () => {
      expect(calculator.multiply(5, 0)).toBe(0);
    });

    it("should multiply by one", () => {
      expect(calculator.multiply(5, 1)).toBe(5);
    });
  });

  describe("divide", () => {
    it("should divide two positive numbers", () => {
      expect(calculator.divide(10, 2)).toBe(5);
    });

    it("should divide with decimal result", () => {
      expect(calculator.divide(5, 2)).toBe(2.5);
    });

    it("should divide negative numbers", () => {
      expect(calculator.divide(-10, 2)).toBe(-5);
      expect(calculator.divide(10, -2)).toBe(-5);
      expect(calculator.divide(-10, -2)).toBe(5);
    });

    it("should throw error for division by zero", () => {
      expect(() => calculator.divide(10, 0)).toThrow("Division by zero");
    });
  });

  describe("isEven", () => {
    it("should return true for even numbers", () => {
      expect(calculator.isEven(2)).toBe(true);
      expect(calculator.isEven(0)).toBe(true);
      expect(calculator.isEven(-2)).toBe(true);
    });

    it("should return false for odd numbers", () => {
      expect(calculator.isEven(1)).toBe(false);
      expect(calculator.isEven(-1)).toBe(false);
      expect(calculator.isEven(3)).toBe(false);
    });
  });

  describe("max", () => {
    it("should return the larger number", () => {
      expect(calculator.max(5, 3)).toBe(5);
      expect(calculator.max(3, 5)).toBe(5);
    });

    it("should return the number when equal", () => {
      expect(calculator.max(5, 5)).toBe(5);
    });

    it("should work with negative numbers", () => {
      expect(calculator.max(-5, -3)).toBe(-3);
      expect(calculator.max(-3, -5)).toBe(-3);
    });
  });

  describe("factorial", () => {
    it("should calculate factorial of 0", () => {
      expect(calculator.factorial(0)).toBe(1);
    });

    it("should calculate factorial of 1", () => {
      expect(calculator.factorial(1)).toBe(1);
    });

    it("should calculate factorial of positive numbers", () => {
      expect(calculator.factorial(5)).toBe(120);
      expect(calculator.factorial(3)).toBe(6);
    });

    it("should throw error for negative numbers", () => {
      expect(() => calculator.factorial(-1)).toThrow(
        "Factorial of negative number"
      );
    });
  });
});
