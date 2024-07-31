pub struct CommonFunction;

impl From<CommonFunction> for common_wit::Target {
    fn from(_: CommonFunction) -> Self {
        common_wit::Target::CommonFunction
    }
}

pub struct CommonFunctionVm;

impl From<CommonFunctionVm> for common_wit::Target {
    fn from(_: CommonFunctionVm) -> Self {
        common_wit::Target::CommonFunctionVm
    }
}
