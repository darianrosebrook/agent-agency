// Test file to verify the todo-analyzer fixes
// This file contains patterns that were broken before the fixes

// TODO: This is a regular TODO that should be detected
function regularFunction() {
  return "hello";
}

// This function has an empty body - should be detected as stub
function emptyFunction() {
}

function main() {
  // work in progress - this should be detected
  console.log("not yet implemented");

  // placeholder code - this should be detected
  return null;
}

// TODO: hardcoded value - this should be detected
const MAGIC_NUMBER = 42;

// temporary implementation - this should be detected
function tempFunc() {
  // quick fix
  return "temporary";
}
