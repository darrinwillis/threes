use rand::prelude::*;

pub fn resolve_rng_from_seed(seed: Option<&mut StdRng>) -> StdRng {
    match seed {
        None => StdRng::from_rng(rand::thread_rng()),
        Some(seed_rng) => StdRng::from_rng(seed_rng),
    }
    .unwrap()
}
