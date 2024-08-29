use common_protos::runtime::runtime_client::RuntimeClient;
use http::Uri;
use tonic_web_wasm_client::Client;

pub fn make_runtime_client(uri: &Uri) -> RuntimeClient<Client> {
    RuntimeClient::new(Client::new(format!(
        "{}://{}",
        uri.scheme_str().unwrap_or_default(),
        uri.authority()
            .map(|authority| authority.as_str())
            .unwrap_or_default()
    )))
}
