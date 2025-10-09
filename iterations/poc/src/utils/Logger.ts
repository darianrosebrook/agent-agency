/**
 * Logger Utility
 *
 * @author @darianrosebrook
 * @description Simple logging utility for the agent agency system
 */

export class Logger {
  private readonly context: string;

  constructor(context: string) {
    this.context = context;
  }

  /**
   * Log an info message
   */
  info(message: string, ...args: unknown[]): void {
    this.log("INFO", message, ...args);
  }

  /**
   * Log a warning message
   */
  warn(message: string, ...args: unknown[]): void {
    this.log("WARN", message, ...args);
  }

  /**
   * Log an error message
   */
  error(message: string, ...args: unknown[]): void {
    this.log("ERROR", message, ...args);
  }

  /**
   * Log a debug message
   */
  debug(message: string, ...args: unknown[]): void {
    this.log("DEBUG", message, ...args);
  }

  /**
   * Internal logging method
   */
  private log(level: string, message: string, ...args: unknown[]): void {
    const timestamp = new Date().toISOString();
    const logMessage = `[${timestamp}] [${level}] [${this.context}] ${message}`;

    if (args.length > 0) {
      console.log(logMessage, ...args);
    } else {
      console.log(logMessage);
    }
  }
}
