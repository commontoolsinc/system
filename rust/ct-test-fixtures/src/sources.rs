//! Source code fixtures

pub mod common {
    //! Source code fixtures that implement 'common:*' targets.

    /// A basic JavaScript Common Module
    pub const BASIC_MODULE_JS: &str = include_str!("../fixtures/common/basic_module.js");

    /// A basic TypeScript Common Module
    pub const BASIC_MODULE_TSX: &str = include_str!("../fixtures/common/basic_module.tsx");

    /// A JavaScript module that returns a stringified
    /// sorted array of all properties on `globalThis`.
    pub const GET_GLOBAL_THIS_PROPS: &str =
        include_str!("../fixtures/common/get_global_this_props.js");

    /// A JavaScript module that returns a stringified
    /// sorted array of all properties on `import.meta`.
    pub const GET_IMPORT_META_PROPS: &str =
        include_str!("../fixtures/common/get_import_meta_props.js");
}
