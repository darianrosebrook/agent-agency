interface User {
  name: string;
  age: number;
}

function processUser(user: any) {
  console.log(user.name.toUpperCase());
  return user.age * 2;
}

const users: User[] = [
  { name: "Alice", age: "30" }, // Type error: age should be number
  { name: "Bob" }, // Type error: missing age property
];

users.forEach((user) => {
  processUser(user); // Type error: user might not have age
});
