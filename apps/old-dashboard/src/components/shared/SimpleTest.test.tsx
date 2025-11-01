import React from "react";
import { render, screen } from "@testing-library/react";
import "@testing-library/jest-dom";

describe("Simple Test", () => {
  it("should render a simple component", () => {
    const TestComponent = () => <div>Hello World</div>;
    render(<TestComponent />);
    expect(screen.getByText("Hello World")).toBeInTheDocument();
  });
});
