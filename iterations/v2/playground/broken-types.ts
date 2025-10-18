// Intentionally broken TypeScript file for arbiter testing
// This file contains multiple compilation errors that the arbiter should fix

interface User {
  id: string;
  name: string;
  email: string;
  // Missing required field: createdAt
}

// Duplicate interface definition (should be removed)
interface User {
  id: string;
  name: string;
  email: string;
  createdAt: Date;
}

// Type mismatch - should be number, not string
const userId: string = 123;

// Missing import
const result = fetchUserData(userId);

// Unused variable
const unusedVar = "this should be removed or prefixed with underscore";

// Function with wrong return type
function calculateTotal(items: number[]): string {
  return items.reduce((sum, item) => sum + item, 0);
}

// Missing error handling
function riskyOperation() {
  const data = JSON.parse(null); // This will throw
  return data;
}

// Inconsistent naming convention
const user_name = "john"; // Should be userName
const userAge = 25; // This is correct

// Missing type annotation
const config = {
  apiUrl: "https://api.example.com",
  timeout: 5000,
  retries: 3
};

// TODO comment that should be addressed
// TODO: Implement proper error handling for API calls

// PLACEHOLDER: This is a placeholder that needs implementation
function placeholderFunction() {
  // PLACEHOLDER: Add actual implementation
}

// MOCK DATA: This should be replaced with real data
const mockUsers = [
  { id: "1", name: "John", email: "john@example.com" },
  { id: "2", name: "Jane", email: "jane@example.com" }
];

export { User, calculateTotal, riskyOperation, config, mockUsers };

