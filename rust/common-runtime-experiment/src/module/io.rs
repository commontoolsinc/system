use std::collections::BTreeMap;

use crate::ValueKind;

#[derive(Default)]
pub struct ModuleIoShape {
    pub input: BTreeMap<String, ValueKind>,
    pub output: BTreeMap<String, ValueKind>,
}
