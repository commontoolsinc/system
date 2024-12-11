#[cfg(target_arch = "wasm32")]
mod inner {
    use std::sync::Once;
    static INITIALIZE_TRACING: Once = Once::new();

    /// Do not call directly.
    /// See [ct_tracing::ct_tracing].
    pub fn ct_tracing_impl() {
        INITIALIZE_TRACING.call_once(|| {
            console_error_panic_hook::set_once();
            tracing_wasm::set_as_global_default();
        })
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod inner {
    use std::sync::Once;
    use tracing::subscriber::SetGlobalDefaultError;
    use tracing_subscriber::{EnvFilter, FmtSubscriber};

    static INITIALIZE_TRACING: Once = Once::new();

    /// Do not call directly.
    /// See [`ct_tracing`].
    pub fn ct_tracing_impl() {
        INITIALIZE_TRACING.call_once(|| {
            if let Err(error) = initialize_tracing_subscriber() {
                println!("Failed to initialize tracing: {}", error);
            }
        });
    }

    fn initialize_tracing_subscriber() -> Result<(), SetGlobalDefaultError> {
        let subscriber = FmtSubscriber::builder()
            .with_env_filter(EnvFilter::from_default_env())
            .finish();
        tracing::subscriber::set_global_default(subscriber)?;
        Ok(())
    }
}

pub use inner::*;
