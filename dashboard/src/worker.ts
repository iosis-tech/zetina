import { loadSwiftnessModule } from "./utils/loadModule";
import type { WorkerMessage, WorkerResponse } from "./utils/types";

self.onmessage = async (event: MessageEvent<WorkerMessage>) => {
  const { proof, layout, commitment } = event.data;

  try {
    // Load the module and verify the proof
    let verify_proof = await loadSwiftnessModule(layout, commitment);
    const [programHash, programOutput] = JSON.parse(verify_proof(proof));

    // Send results back to the main thread
    const response: WorkerResponse = { programHash, programOutput };
    self.postMessage(response);
  } catch (error) {
    // Send error back to the main thread
    const response: WorkerResponse = { error: (error as Error).message };
    self.postMessage(response);
  }
};
