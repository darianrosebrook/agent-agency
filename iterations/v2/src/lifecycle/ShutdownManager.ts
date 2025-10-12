/**
 * Graceful Shutdown Manager
 *
 * Coordinates graceful shutdown of all system components.
 * Handles SIGTERM/SIGINT signals and ensures clean resource cleanup.
 *
 * @author @darianrosebrook
 */

export type ShutdownHook = () => Promise<void>;

export interface ShutdownOptions {
  shutdownTimeout?: number; // Max time to wait for shutdown (ms)
  forceExit?: boolean; // Force exit after timeout
}

/**
 * Manages graceful shutdown of the application
 *
 * Registers shutdown hooks that are executed in order when
 * the application receives a termination signal.
 */
export class ShutdownManager {
  private hooks: Array<{ name: string; hook: ShutdownHook }> = [];
  private isShuttingDown = false;
  private shutdownTimeout: number;
  private forceExit: boolean;

  constructor(options: ShutdownOptions = {}) {
    this.shutdownTimeout = options.shutdownTimeout || 30000; // 30 seconds default
    this.forceExit = options.forceExit !== false; // true by default
    this.registerSignalHandlers();
  }

  /**
   * Register a shutdown hook
   *
   * Hooks are executed in the order they are registered.
   */
  registerShutdownHook(name: string, hook: ShutdownHook): void {
    this.hooks.push({ name, hook });
  }

  /**
   * Unregister a shutdown hook by name
   */
  unregisterShutdownHook(name: string): void {
    this.hooks = this.hooks.filter((h) => h.name !== name);
  }

  /**
   * Register signal handlers for graceful shutdown
   */
  private registerSignalHandlers(): void {
    process.on("SIGTERM", () => this.shutdown("SIGTERM"));
    process.on("SIGINT", () => this.shutdown("SIGINT"));

    // Handle uncaught errors
    process.on("uncaughtException", (error) => {
      console.error("Uncaught exception:", error);
      this.shutdown("uncaughtException");
    });

    process.on("unhandledRejection", (reason, promise) => {
      console.error("Unhandled rejection at:", promise, "reason:", reason);
      this.shutdown("unhandledRejection");
    });
  }

  /**
   * Execute graceful shutdown
   */
  private async shutdown(signal: string): Promise<void> {
    if (this.isShuttingDown) {
      console.warn(`Shutdown already in progress (triggered by ${signal})`);
      return;
    }

    this.isShuttingDown = true;
    console.log(`Received ${signal}, starting graceful shutdown...`);

    const shutdownPromise = this.executeShutdownHooks();
    const timeoutPromise = this.createTimeoutPromise();

    try {
      await Promise.race([shutdownPromise, timeoutPromise]);
      console.log("Graceful shutdown complete");
    } catch (error) {
      console.error("Error during shutdown:", error);
    } finally {
      if (this.forceExit) {
        process.exit(0);
      }
    }
  }

  /**
   * Execute all registered shutdown hooks
   */
  private async executeShutdownHooks(): Promise<void> {
    console.log(`Executing ${this.hooks.length} shutdown hooks...`);

    for (const { name, hook } of this.hooks) {
      try {
        console.log(`Executing shutdown hook: ${name}`);
        await hook();
        console.log(`✅ Shutdown hook complete: ${name}`);
      } catch (error) {
        console.error(`❌ Error executing shutdown hook '${name}':`, error);
        // Continue with other hooks even if one fails
      }
    }
  }

  /**
   * Create timeout promise for shutdown
   */
  private createTimeoutPromise(): Promise<void> {
    return new Promise<void>((resolve) =>
      setTimeout(() => {
        console.warn(
          `Shutdown timeout (${this.shutdownTimeout}ms) reached, forcing exit`
        );
        resolve();
      }, this.shutdownTimeout)
    );
  }

  /**
   * Trigger manual shutdown (for testing)
   */
  async triggerShutdown(): Promise<void> {
    await this.shutdown("manual");
  }

  /**
   * Check if shutdown is in progress
   */
  isShutdown(): boolean {
    return this.isShuttingDown;
  }

  /**
   * Get list of registered shutdown hooks
   */
  getRegisteredHooks(): string[] {
    return this.hooks.map((h) => h.name);
  }

  /**
   * Clear all shutdown hooks (for testing)
   */
  clearHooks(): void {
    this.hooks = [];
  }
}

