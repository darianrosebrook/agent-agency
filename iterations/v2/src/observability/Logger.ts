/**
 * Simple Logger Implementation
 *
 * Basic logging utility for the system.
 *
 * @author @darianrosebrook
 */

export enum LogLevel {
  _ERROR = 0,
  _WARN = 1,
  _INFO = 2,
  _DEBUG = 3,
}

export class Logger {
  private level: LogLevel;
  private name: string;

  constructor(name: string, level: LogLevel = LogLevel.INFO) {
    this.name = name;
    this.level = level;
  }

  public error(message: string, ...args: any[]): void {
    if (this.level >= LogLevel.ERROR) {
      console.error(
        `[${new Date().toISOString()}] ERROR [${this.name}] ${message}`,
        ...args
      );
    }
  }

  public warn(message: string, ...args: any[]): void {
    if (this.level >= LogLevel.WARN) {
      console.warn(
        `[${new Date().toISOString()}] WARN [${this.name}] ${message}`,
        ...args
      );
    }
  }

  public info(message: string, ...args: any[]): void {
    if (this.level >= LogLevel.INFO) {
      console.info(
        `[${new Date().toISOString()}] INFO [${this.name}] ${message}`,
        ...args
      );
    }
  }

  public debug(message: string, ...args: any[]): void {
    if (this.level >= LogLevel.DEBUG) {
      console.debug(
        `[${new Date().toISOString()}] DEBUG [${this.name}] ${message}`,
        ...args
      );
    }
  }

  public setLevel(level: LogLevel): void {
    this.level = level;
  }

  public getLevel(): LogLevel {
    return this.level;
  }
}
