/**
 * GSAP Mock for Jest Testing
 * Mocks GSAP functions to avoid ES6 module issues during testing
 */

export const gsap = {
  set: jest.fn().mockReturnValue({
    to: jest.fn().mockReturnValue({
      ease: jest.fn()
    })
  }),
  to: jest.fn().mockReturnValue({
    ease: jest.fn()
  }),
  from: jest.fn().mockReturnValue({
    ease: jest.fn()
  }),
  timeline: jest.fn().mockReturnValue({
    to: jest.fn(),
    from: jest.fn(),
    set: jest.fn(),
    add: jest.fn(),
    play: jest.fn(),
    pause: jest.fn(),
    restart: jest.fn()
  }),
  registerPlugin: jest.fn(),
  registerEffect: jest.fn(),
  utils: {
    wrap: jest.fn(),
    random: jest.fn(),
    mapRange: jest.fn(),
    interpolate: jest.fn()
  }
};

export default gsap;

