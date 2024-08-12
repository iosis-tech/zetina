import init_swiftness_dex_blake2s, {
  verify_proof as verify_proof_swiftness_dex_blake2s,
} from "swiftness-dex-blake2s";
import init_swiftness_dex_keccak, {
  verify_proof as verify_proof_swiftness_dex_keccak,
} from "swiftness-dex-keccak";
import init_swiftness_recursive_blake2s, {
  verify_proof as verify_proof_swiftness_recursive_blake2s,
} from "swiftness-recursive-blake2s";
import init_swiftness_recursive_keccak, {
  verify_proof as verify_proof_swiftness_recursive_keccak,
} from "swiftness-recursive-keccak";
import init_swiftness_recursive_with_poseidon_blake2s, {
  verify_proof as verify_proof_swiftness_recursive_with_poseidon_blake2s,
} from "swiftness-recursive-with-poseidon-blake2s";
import init_swiftness_recursive_with_poseidon_keccak, {
  verify_proof as verify_proof_swiftness_recursive_with_poseidon_keccak,
} from "swiftness-recursive-with-poseidon-keccak";
import init_swiftness_small_blake2s, {
  verify_proof as verify_proof_swiftness_small_blake2s,
} from "swiftness-small-blake2s";
import init_swiftness_small_keccak, {
  verify_proof as verify_proof_swiftness_small_keccak,
} from "swiftness-small-keccak";
import init_swiftness_starknet_blake2s, {
  verify_proof as verify_proof_swiftness_starknet_blake2s,
} from "swiftness-starknet-blake2s";
import init_swiftness_starknet_keccak, {
  verify_proof as verify_proof_swiftness_starknet_keccak,
} from "swiftness-starknet-keccak";
import init_swiftness_starknet_with_keccak_blake2s, {
  verify_proof as verify_proof_swiftness_starknet_with_keccak_blake2s,
} from "swiftness-starknet-with-keccak-blake2s";
import init_swiftness_starknet_with_keccak_keccak, {
  verify_proof as verify_proof_swiftness_starknet_with_keccak_keccak,
} from "swiftness-starknet-with-keccak-keccak";

export enum Layout {
  DEX = "dex",
  RECURSIVE = "recursive",
  RECURSIVE_WITH_POSEIDON = "recursive_with_poseidon",
  SMALL = "small",
  STARKNET = "starknet",
  STARKNET_WITH_KECCAK = "starknet_with_keccak",
}

export const matchLayout = (layout: string): Layout | undefined => {
  switch (layout) {
    case Layout.DEX:
      return Layout.DEX;
    case Layout.RECURSIVE:
      return Layout.RECURSIVE;
    case Layout.RECURSIVE_WITH_POSEIDON:
      return Layout.RECURSIVE_WITH_POSEIDON;
    case Layout.SMALL:
      return Layout.SMALL;
    case Layout.STARKNET:
      return Layout.STARKNET;
    case Layout.STARKNET_WITH_KECCAK:
      return Layout.STARKNET_WITH_KECCAK;
    default:
      return undefined;
  }
};
export enum Commitment {
  BLAKE2S = "blake256",
  KECCAK = "keccak256",
}

export const matchCommitment = (commitment: string): Commitment | undefined => {
  switch (commitment) {
    case Commitment.BLAKE2S:
      return Commitment.BLAKE2S;
    case Commitment.KECCAK:
      return Commitment.KECCAK;
    default:
      return undefined;
  }
};

type VerifierFunctionTuple = [() => Promise<any>, (...args: any[]) => string];
type VerifierMap = {
  [key in `${Layout}_${Commitment}`]?: VerifierFunctionTuple;
};

const verifier_map: VerifierMap = {
  [`${Layout.DEX}_${Commitment.BLAKE2S}`]: [
    init_swiftness_dex_blake2s,
    verify_proof_swiftness_dex_blake2s,
  ],
  [`${Layout.DEX}_${Commitment.KECCAK}`]: [
    init_swiftness_dex_keccak,
    verify_proof_swiftness_dex_keccak,
  ],
  [`${Layout.RECURSIVE}_${Commitment.BLAKE2S}`]: [
    init_swiftness_recursive_blake2s,
    verify_proof_swiftness_recursive_blake2s,
  ],
  [`${Layout.RECURSIVE}_${Commitment.KECCAK}`]: [
    init_swiftness_recursive_keccak,
    verify_proof_swiftness_recursive_keccak,
  ],
  [`${Layout.RECURSIVE_WITH_POSEIDON}_${Commitment.BLAKE2S}`]: [
    init_swiftness_recursive_with_poseidon_blake2s,
    verify_proof_swiftness_recursive_with_poseidon_blake2s,
  ],
  [`${Layout.RECURSIVE_WITH_POSEIDON}_${Commitment.KECCAK}`]: [
    init_swiftness_recursive_with_poseidon_keccak,
    verify_proof_swiftness_recursive_with_poseidon_keccak,
  ],
  [`${Layout.SMALL}_${Commitment.BLAKE2S}`]: [
    init_swiftness_small_blake2s,
    verify_proof_swiftness_small_blake2s,
  ],
  [`${Layout.SMALL}_${Commitment.KECCAK}`]: [
    init_swiftness_small_keccak,
    verify_proof_swiftness_small_keccak,
  ],
  [`${Layout.STARKNET}_${Commitment.BLAKE2S}`]: [
    init_swiftness_starknet_blake2s,
    verify_proof_swiftness_starknet_blake2s,
  ],
  [`${Layout.STARKNET}_${Commitment.KECCAK}`]: [
    init_swiftness_starknet_keccak,
    verify_proof_swiftness_starknet_keccak,
  ],
  [`${Layout.STARKNET_WITH_KECCAK}_${Commitment.BLAKE2S}`]: [
    init_swiftness_starknet_with_keccak_blake2s,
    verify_proof_swiftness_starknet_with_keccak_blake2s,
  ],
  [`${Layout.STARKNET_WITH_KECCAK}_${Commitment.KECCAK}`]: [
    init_swiftness_starknet_with_keccak_keccak,
    verify_proof_swiftness_starknet_with_keccak_keccak,
  ],
};

// Function to dynamically import the required swiftness package
export async function loadSwiftnessModule(
  layout: Layout,
  commitment: Commitment,
) {
  const key = `${layout}_${commitment}` as keyof VerifierMap;
  if (verifier_map[key]) {
    const [init, verify] = verifier_map[key]!;
    await init();
    return verify;
  } else {
    throw new Error("Invalid layout or commitment type");
  }
}
