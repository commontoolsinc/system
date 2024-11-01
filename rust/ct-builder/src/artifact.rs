use ct_protos::builder::ReadComponentResponse;
use deno_emit::BundleEmit;

pub struct Artifact {
    pub component: String,
    pub source_map: Option<String>,
}

impl From<BundleEmit> for Artifact {
    fn from(value: BundleEmit) -> Self {
        Artifact {
            component: value.code,
            source_map: value.maybe_map,
        }
    }
}

impl From<Artifact> for ReadComponentResponse {
    fn from(value: Artifact) -> Self {
        ReadComponentResponse {
            component: value.component,
            source_map: value.source_map,
        }
    }
}

impl From<ReadComponentResponse> for Artifact {
    fn from(value: ReadComponentResponse) -> Self {
        Artifact {
            component: value.component,
            source_map: value.source_map,
        }
    }
}
