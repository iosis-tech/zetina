import { z } from "zod";

// Zod  for DelegateRequest
export const DelegateRequest = z.object({
  cairo_pie: z.instanceof(Uint8Array),
});
export type DelegateRequest = z.infer<typeof DelegateRequest>;

// Zod  for DelegateResponse
export const DelegateResponse = z.object({
  job_hash: z.number(),
});
export type DelegateResponse = z.infer<typeof DelegateResponse>;

// Zod  for JobEventsRequest
export const JobEventsRequest = z.object({
  job_hash: z.number(),
});
export type JobEventsRequest = z.infer<typeof JobEventsRequest>;

// Zod  for JobEventsResponse
export const Picked = z.object({
  type: z.literal("Picked"),
  data: z.number(),
});
export type Picked = z.infer<typeof Picked>;

export const Witness = z.object({
  type: z.literal("Witness"),
  data: z.instanceof(Uint8Array),
});
export type Witness = z.infer<typeof Witness>;

export const JobEventsResponse = z.union([Picked, Witness]);
export type JobEventsResponse = z.infer<typeof JobEventsResponse>;
