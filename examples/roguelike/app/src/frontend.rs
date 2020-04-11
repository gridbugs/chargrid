#[derive(Debug, Clone, Copy)]
pub enum Frontend {
    Web,
    AnsiTerminal,
    Graphical,
}

impl Frontend {
    pub(crate) fn log_rng_seed(self, seed: u64) {
        match self {
            Self::Web => log::info!("RNG Seed: {}", seed),
            Self::AnsiTerminal => (),
            Self::Graphical => println!("RNG Seed: {}", seed),
        }
    }
}
