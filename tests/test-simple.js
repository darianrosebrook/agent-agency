// Simple test to verify Jest works
describe("Basic Test Infrastructure", () => {
  test("should pass a simple test", () => {
    expect(1 + 1).toBe(2);
  });

  test("should handle async operations", async () => {
    const result = await Promise.resolve("test");
    expect(result).toBe("test");
  });
});
