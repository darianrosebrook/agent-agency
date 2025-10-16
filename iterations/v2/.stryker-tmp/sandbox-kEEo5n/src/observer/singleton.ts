// @ts-nocheck
import { ObserverBridge } from "./ObserverBridge";

let instance: ObserverBridge | null = null;

export function setObserverBridge(bridge: ObserverBridge | null): void {
  instance = bridge;
}

export function getObserverBridge(): ObserverBridge | null {
  return instance;
}

