/**
 * @fileoverview Simple Mutex implementation for thread-safe operations (ARBITER-005)
 *
 * Provides mutual exclusion for critical sections in async operations.
 *
 * @author @darianrosebrook
 */
// @ts-nocheck


/**
 * Simple Mutex for async operations
 */
export class Mutex {
  private locked: boolean = false;
  private waitingQueue: Array<() => void> = [];

  /**
   * Acquire the lock
   */
  async acquire(): Promise<void> {
    return new Promise((resolve) => {
      if (!this.locked) {
        this.locked = true;
        resolve();
      } else {
        this.waitingQueue.push(() => {
          this.locked = true;
          resolve();
        });
      }
    });
  }

  /**
   * Release the lock
   */
  release(): void {
    if (this.waitingQueue.length > 0) {
      const next = this.waitingQueue.shift();
      if (next) {
        next();
      }
    } else {
      this.locked = false;
    }
  }

  /**
   * Execute a function with the lock held
   */
  async withLock<T>(fn: () => Promise<T>): Promise<T> {
    await this.acquire();
    try {
      return await fn();
    } finally {
      this.release();
    }
  }

  /**
   * Check if the mutex is currently locked
   */
  isLocked(): boolean {
    return this.locked;
  }

  /**
   * Get the number of waiting operations
   */
  getWaitingCount(): number {
    return this.waitingQueue.length;
  }
}

/**
 * Read-Write Mutex for operations that can be concurrent for reads
 */
export class RWMutex {
  private readers: number = 0;
  private writer: boolean = false;
  private waitingReaders: Array<() => void> = [];
  private waitingWriters: Array<() => void> = [];

  /**
   * Acquire read lock
   */
  async acquireRead(): Promise<void> {
    return new Promise((resolve) => {
      if (!this.writer && this.waitingWriters.length === 0) {
        this.readers++;
        resolve();
      } else {
        this.waitingReaders.push(() => {
          this.readers++;
          resolve();
        });
      }
    });
  }

  /**
   * Acquire write lock
   */
  async acquireWrite(): Promise<void> {
    return new Promise((resolve) => {
      if (this.readers === 0 && !this.writer) {
        this.writer = true;
        resolve();
      } else {
        this.waitingWriters.push(() => {
          this.writer = true;
          resolve();
        });
      }
    });
  }

  /**
   * Release read lock
   */
  releaseRead(): void {
    this.readers--;
    if (this.readers === 0 && this.waitingWriters.length > 0) {
      const next = this.waitingWriters.shift();
      if (next) {
        next();
      }
    }
  }

  /**
   * Release write lock
   */
  releaseWrite(): void {
    this.writer = false;
    // Prioritize writers over readers
    if (this.waitingWriters.length > 0) {
      const next = this.waitingWriters.shift();
      if (next) {
        next();
      }
    } else if (this.waitingReaders.length > 0) {
      // Wake up all waiting readers
      const readers = [...this.waitingReaders];
      this.waitingReaders = [];
      readers.forEach((reader) => reader());
    }
  }

  /**
   * Execute a read operation with lock held
   */
  async withReadLock<T>(fn: () => Promise<T>): Promise<T> {
    await this.acquireRead();
    try {
      return await fn();
    } finally {
      this.releaseRead();
    }
  }

  /**
   * Execute a write operation with lock held
   */
  async withWriteLock<T>(fn: () => Promise<T>): Promise<T> {
    await this.acquireWrite();
    try {
      return await fn();
    } finally {
      this.releaseWrite();
    }
  }

  /**
   * Check if currently writing
   */
  isWriting(): boolean {
    return this.writer;
  }

  /**
   * Get current reader count
   */
  getReaderCount(): number {
    return this.readers;
  }

  /**
   * Get waiting reader count
   */
  getWaitingReaderCount(): number {
    return this.waitingReaders.length;
  }

  /**
   * Get waiting writer count
   */
  getWaitingWriterCount(): number {
    return this.waitingWriters.length;
  }
}
