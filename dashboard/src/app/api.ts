import { z } from "zod";

const hexStringSchema = z
  .string()
  .regex(/^[0-9a-fA-F]+$/, "Invalid hex string");
const base58Pattern =
  /^[123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz]+$/;
const base58Schema = z.string().regex(base58Pattern, "Invalid Base58 string");
const bytesSchema = z.array(z.number());

// Zod  for DelegateRequest
export const DelegateRequest = z.object({
  pie: bytesSchema,
});
export type DelegateRequest = z.infer<typeof DelegateRequest>;

// Zod  for DelegateResponse
export const DelegateResponse = z.object({
  job_key: hexStringSchema,
});
export type DelegateResponse = z.infer<typeof DelegateResponse>;

// Zod  for JobEventsRequest
export const JobEventsRequest = z.object({
  job_key: hexStringSchema,
});
export type JobEventsRequest = z.infer<typeof JobEventsRequest>;

export const JobEventsResponse = z.object({
  type: z
    .literal("Finished")
    .or(z.literal("Delegated"))
    .or(z.literal("BidReceived"))
    .or(z.literal("Propagated")),
  data: z.any(),
});
export type JobEventsResponse = z.infer<typeof JobEventsResponse>;

export const Proof = bytesSchema;
export type Proof = z.infer<typeof Proof>;

export const PeerId = base58Schema;
export type PeerId = z.infer<typeof PeerId>;
