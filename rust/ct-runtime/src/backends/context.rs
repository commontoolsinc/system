use rand::Rng;

/// A context providing host functionality for linking
/// WASI in backends.
///
/// Currently only providng random number generator.
pub struct Context {
    random: cap_rand::rngs::StdRng,
}

impl Context {
    /// Implements `wasi:random/random@0.2.0` `get-random-bytes`.
    pub fn get_random_bytes(&mut self, len: u64) -> Vec<u8> {
        (&mut self.random)
            .sample_iter(cap_rand::distributions::Standard)
            .take(len as usize)
            .collect()
    }
}

impl Context {
    /// Create a new [`Context`].
    pub fn new() -> Self {
        let random = thread_rng();
        Self { random }
    }
}

impl Default for Context {
    fn default() -> Self {
        Context::new()
    }
}

fn thread_rng() -> cap_rand::rngs::StdRng {
    use cap_rand::{Rng, SeedableRng};
    let mut rng = cap_rand::thread_rng(cap_rand::ambient_authority());
    cap_rand::rngs::StdRng::from_seed(rng.gen())
}
