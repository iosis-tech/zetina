import { z } from "zod";

// Zod  for DelegateRequest
export const DelegateRequest = z.object({
  pie: z.array(z.number()),
});
export type DelegateRequest = z.infer<typeof DelegateRequest>;

// Zod  for DelegateResponse
export const DelegateResponse = z.object({
  job_hash: z.coerce.bigint(),
});
export type DelegateResponse = z.infer<typeof DelegateResponse>;

// Zod  for JobEventsRequest
export const JobEventsRequest = z.object({
  job_hash: z.coerce.bigint(),
});
export type JobEventsRequest = z.infer<typeof JobEventsRequest>;

export const JobEventsResponse = z.object({
  type: z.literal("Finished"),
  data: z.any(),
});
export type JobEventsResponse = z.infer<typeof JobEventsResponse>;

export const JobHash = z.coerce.bigint();
export type JobHash = z.infer<typeof JobHash>;

export const Proof = z.array(z.number());
export type Proof = z.infer<typeof Proof>;
