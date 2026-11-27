import { timestamp as currentTime } from './clock';
export function observedAt(): number {
  return currentTime();
}
