export type Answer = { no: number; verdict: string };
export const isDuplicate = (a: Answer, b: Answer) => a.no === b.no;
import { handler } from "../adapters/http";   // わざと違反
