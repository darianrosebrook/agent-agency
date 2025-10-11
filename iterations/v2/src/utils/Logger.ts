/**
 * Simple Logger Utility
 *
 * Basic logging functionality for the Agent Registry system.
 *
 * @author @darianrosebrook
 */

export enum LogLevel {
  DEBUG = 0,
  INFO = 1,
  WARN = 2,
  ERROR = 3,
}

export class Logger {
  private name: string;
  private level: LogLevel;

  constructor(name: string, level: LogLevel = LogLevel.INFO) {
    this.name = name;
    this.level = level;
  }

  debug(message: string, data?: any): void {
    if (this.level <= LogLevel.DEBUG) {
      console.debug(`[${this.name}] DEBUG: ${message}`, data || "");
    }
  }

  info(message: string, data?: any): void {
    if (this.level <= LogLevel.INFO) {
      console.info(`[${this.name}] INFO: ${message}`, data || "");
    }
  }

  warn(message: string, data?: any): void {
    if (this.level <= LogLevel.WARN) {
      console.warn(`[${this.name}] WARN: ${message}`, data || "");
    }
  }

  error(message: string, data?: any): void {
    if (this.level <= LogLevel.ERROR) {
      console.error(`[${this.name}] ERROR: ${message}`, data || "");
    }
  }

  setLevel(level: LogLevel): void {
    this.level = level;
  }
}
