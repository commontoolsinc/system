use std::sync::LazyLock;

static INITIALIZERS: LazyLock<()> = LazyLock::new(|| {
    console_error_panic_hook::set_once();
    tracing_wasm::set_as_global_default();
    ()
});

pub fn global_initializers() {
    LazyLock::force(&INITIALIZERS);
}
