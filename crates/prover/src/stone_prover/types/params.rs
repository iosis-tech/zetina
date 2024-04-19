use serde::Serialize;

#[derive(Debug, PartialEq, Serialize, Default)]
pub struct CachedLdeConfig {
    pub store_full_lde: bool,
    pub use_fft_for_eval: bool,
}

#[derive(Debug, PartialEq, Serialize)]
pub struct Params {
    pub cached_lde_config: CachedLdeConfig,
    pub constraint_polynomial_task_size: u32,
    pub n_out_of_memory_merkle_layers: u32,
    pub table_prover_n_tasks_per_segment: u32,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            cached_lde_config: CachedLdeConfig::default(),
            constraint_polynomial_task_size: 256,
            n_out_of_memory_merkle_layers: 0,
            table_prover_n_tasks_per_segment: 32,
        }
    }
}
