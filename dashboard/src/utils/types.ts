import { Layout, Commitment } from "@/utils/loadModule";

export interface WorkerMessage {
  proof: string;
  layout: Layout;
  commitment: Commitment;
}

export interface WorkerResponse {
  programHash?: string;
  programOutput?: string;
  error?: string;
}
