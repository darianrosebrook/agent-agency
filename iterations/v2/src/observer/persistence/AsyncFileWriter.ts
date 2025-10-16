import fs from "fs";
import path from "path";
import { once } from "events";
import { ObserverConfig } from "../types";

// Node.js types
type NodeJS_Timeout = ReturnType<typeof setTimeout>;

interface WriteTask {
  payload: string;
  resolve: () => void;
  reject: (error: unknown) => void;
}

/**
 * AsyncFileWriter batches JSONL writes onto a single write stream. It enforces
 * rotation by size and ensures fsync occurs periodically for durability.
 */
export class AsyncFileWriter {
  private readonly config: ObserverConfig;
  private readonly prefix: string;
  private stream?: fs.WriteStream;
  private queue: WriteTask[] = [];
  private flushing = false;
  private flushTimer?: NodeJS_Timeout;
  private bytesWrittenSinceSync = 0;
  private currentFilePath?: string;
  private currentFileSize = 0;
  private currentDate = this.getDateStamp();

  constructor(config: ObserverConfig, prefix: string) {
    this.config = config;
    this.prefix = prefix;
  }

  append(line: object): Promise<void> {
    const payload = JSON.stringify(line) + "\n";
    return new Promise<void>((resolve, reject) => {
      this.queue.push({ payload, resolve, reject });
      if (this.queue.length >= 64) {
        this.flush();
      } else if (!this.flushTimer) {
        this.flushTimer = setTimeout(() => this.flush(), this.config.flushIntervalMs);
      }
    });
  }

  async flush(force = false): Promise<void> {
    if (this.flushing) {
      return;
    }
    if (!force && this.queue.length === 0) {
      return;
    }
    this.flushing = true;
    if (this.flushTimer) {
      clearTimeout(this.flushTimer);
      this.flushTimer = undefined;
    }

    try {
      await this.ensureStream();
      while (this.queue.length) {
        const batch = this.queue.splice(0, 128);
        const chunk = batch.map((task) => task.payload).join("");

        if (this.shouldRotate(chunk.length)) {
          await this.rotate();
          await this.ensureStream();
        }

        if (!this.stream) {
          throw new Error("Write stream unavailable");
        }

        const writeSucceeded = this.stream.write(chunk);
        this.currentFileSize += Buffer.byteLength(chunk, "utf8");
        this.bytesWrittenSinceSync += Buffer.byteLength(chunk, "utf8");

        if (!writeSucceeded) {
          await once(this.stream, "drain");
        }

        batch.forEach((task) => task.resolve());

        if (this.bytesWrittenSinceSync >= 1024 * 64) {
          await this.fsync();
          this.bytesWrittenSinceSync = 0;
        }
      }
    } catch (error) {
      // reject all pending tasks
      for (const task of this.queue.splice(0, this.queue.length)) {
        task.reject(error);
      }
      throw error;
    } finally {
      this.flushing = false;
    }
  }

  async close(): Promise<void> {
    if (this.flushTimer) {
      clearTimeout(this.flushTimer);
      this.flushTimer = undefined;
    }
    await this.flush(true);
    if (this.stream) {
      await new Promise<void>((resolve) => this.stream!.end(resolve));
      this.stream = undefined;
    }
  }

  getActiveFile(): string | undefined {
    return this.currentFilePath;
  }

  private async ensureStream(): Promise<void> {
    if (this.stream) {
      return;
    }
    const dateStamp = this.getDateStamp();
    if (dateStamp !== this.currentDate) {
      this.currentDate = dateStamp;
      await this.rotate();
    }

    const filePath = this.computeFilePath();
    fs.mkdirSync(path.dirname(filePath), { recursive: true });
    this.stream = fs.createWriteStream(filePath, { flags: "a" });
    this.currentFilePath = filePath;
    this.currentFileSize = fs.existsSync(filePath)
      ? fs.statSync(filePath).size
      : 0;
  }

  private async rotate(): Promise<void> {
    if (this.stream) {
      await new Promise<void>((resolve) => this.stream!.end(resolve));
      this.stream = undefined;
    }
    this.bytesWrittenSinceSync = 0;
    this.currentFilePath = undefined;
    this.currentFileSize = 0;
  }

  private computeFilePath(): string {
    const dir = path.join(this.config.dataDir, this.prefix);
    fs.mkdirSync(dir, { recursive: true });
    const base = `${this.prefix}-${this.currentDate}`;
    let index = 0;
    let candidate = path.join(dir, `${base}-${index}.jsonl`);
    while (fs.existsSync(candidate) && fs.statSync(candidate).size >= this.maxBytes()) {
      index += 1;
      candidate = path.join(dir, `${base}-${index}.jsonl`);
    }
    return candidate;
  }

  private shouldRotate(incomingBytes: number): boolean {
    return this.currentFileSize + incomingBytes > this.maxBytes();
  }

  private maxBytes(): number {
    return this.config.rotateMB * 1024 * 1024;
  }

  private async fsync(): Promise<void> {
    if (!this.stream) return;
    const fd = (this.stream as unknown as { fd?: number }).fd;
    if (typeof fd !== "number" || fd < 0) {
      return;
    }
    await new Promise<void>((resolve, reject) => {
      fs.fsync(fd, (error) => {
        if (error) reject(error);
        else resolve();
      });
    });
  }

  private getDateStamp(): string {
    const now = new Date();
    return `${now.getUTCFullYear()}-${String(now.getUTCMonth() + 1).padStart(
      2,
      "0"
    )}-${String(now.getUTCDate()).padStart(2, "0")}`;
  }
}
