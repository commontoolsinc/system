use crate::Error;
use ct_common::ContentType;

static JAVASCRIPT_VM: &[u8] = include_bytes!(env!("CT_JS_VM_WASM_PATH"));

/// Virtual Machines for various languages supported.
#[derive(PartialEq, Eq, Hash)]
pub enum VirtualMachine {
    /// JavaScript virtual machine.
    JavaScript,
}

impl VirtualMachine {
    /// The bytes of the specified [`VirtualMachine`].
    pub fn as_bytes(&self) -> &[u8] {
        match self {
            VirtualMachine::JavaScript => JAVASCRIPT_VM,
        }
    }
}

impl TryFrom<&ContentType> for VirtualMachine {
    type Error = Error;
    fn try_from(value: &ContentType) -> Result<Self, Self::Error> {
        match value {
            ContentType::JavaScript => Ok(VirtualMachine::JavaScript),
        }
    }
}
