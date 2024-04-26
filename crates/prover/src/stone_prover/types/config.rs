use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Default, Deserialize)]
pub struct CachedLdeConfig {
    pub store_full_lde: bool,
    pub use_fft_for_eval: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Config {
    pub cached_lde_config: CachedLdeConfig,
    pub constraint_polynomial_task_size: u64,
    pub n_out_of_memory_merkle_layers: u64,
    pub table_prover_n_tasks_per_segment: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            cached_lde_config: CachedLdeConfig::default(),
            constraint_polynomial_task_size: 256,
            n_out_of_memory_merkle_layers: 0,
            table_prover_n_tasks_per_segment: 32,
        }
    }
}
