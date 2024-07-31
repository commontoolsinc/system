// Targets

use std::marker::PhantomData;

struct CommonFunction;

struct CommonFunctionVm;

// Schedule

struct PrefersLocal;

struct PrefersRemote;

// Module

struct Module<T, S, R> {
    target: PhantomData<T>,
    schedule: PhantomData<S>,

    runtime: R,
}

// Instantiable

trait Instantiable {
    type ModuleInstance;

    fn instantiate(&mut self) -> Self::ModuleInstance;
}

impl<R> Instantiable for Module<CommonFunction, PrefersLocal, R>
where
    R: LocalInstantiation + NativeMachine,
{
    type ModuleInstance = LocalFunction;

    fn instantiate(&mut self) -> Self::ModuleInstance {
        LocalFunction {}
    }
}

impl<R> Instantiable for Module<CommonFunction, PrefersRemote, R>
where
    R: RemoteInstantiation + NativeMachine,
{
    type ModuleInstance = RemoteFunction;

    fn instantiate(&mut self) -> Self::ModuleInstance {
        RemoteFunction {}
    }
}

impl<R> Instantiable for Module<CommonFunctionVm, PrefersLocal, R>
where
    R: LocalInstantiation + VirtualMachine,
{
    type ModuleInstance = LocalFunctionScript;

    fn instantiate(&mut self) -> Self::ModuleInstance {
        LocalFunctionScript {}
    }
}

impl<R> Instantiable for Module<CommonFunctionVm, PrefersRemote, R>
where
    R: RemoteInstantiation + VirtualMachine,
{
    type ModuleInstance = RemoteFunctionScript;

    fn instantiate(&mut self) -> Self::ModuleInstance {
        RemoteFunctionScript {}
    }
}

// Module

pub trait FunctionInstance {
    fn run(&self) {}
}

pub struct LocalFunction {}
impl FunctionInstance for LocalFunction {}

pub struct RemoteFunction {}
impl FunctionInstance for RemoteFunction {}

pub struct LocalFunctionScript {}
impl FunctionInstance for LocalFunctionScript {}

pub struct RemoteFunctionScript {}
impl FunctionInstance for RemoteFunctionScript {}

// Runtime Capabilities

pub trait LocalInstantiation {}

pub trait RemoteInstantiation {}

pub trait NativeMachine {}

pub trait VirtualMachine {}

// use std::collections::BTreeMap;
// use std::marker::PhantomData;

// use common_wit::Target;

// use crate::{ModuleSource, Schedule};
// use crate::{SourceCode, ValueKind};

// struct CommonFunction;

// impl From<CommonFunction> for Target {
//     fn from(_: CommonFunction) -> Self {
//         Target::CommonModule
//     }
// }

// struct CommonFunctionScript;

// impl From<CommonFunctionScript> for Target {
//     fn from(value: CommonFunctionScript) -> Self {
//         Target::CommonScript
//     }
// }

// struct LocalSchedule;

// struct RemoteSchedule;

// pub trait Runtime<I> {
//     fn inst(&mut self) -> I;
// }

// pub trait LocalRuntime<I>: Runtime<I> {}

// pub trait RemoteRuntime<I>: Runtime<I> {}

// pub trait NativeEnvironmentRuntime<I>: Runtime<I> {}
// pub trait VirtualEnvironmentRuntime<I>: Runtime<I> {}

// pub trait CommonFunctionInstance {}
// pub trait CommonFunctionScriptInstance {}

// pub struct Module<T, S, R>
// where
//     T: Into<Target>,
//     S: Into<Schedule>,
//     R: Runtime,
// {
//     target: T,
//     schedule: S,
//     runtime: R,

//     source: ModuleSource,
//     inputs: BTreeMap<String, ValueKind>,
//     outputs: BTreeMap<String, ValueKind>,
// }

// impl<R, M> Module<CommonFunction, LocalSchedule, R>
// where
//     R: LocalRuntime<M>,
// {
//     pub async fn instantiate() -> M;
// }

// #[derive(Default)]
// pub struct ModuleBuilder<Target, Schedule, Runtime> {
//     target: Option<Target>,
//     schedule: Option<Schedule>,
//     runtime: Option<Runtime>,
//     source_code: BTreeMap<String, SourceCode>,
//     inputs: BTreeMap<String, ValueKind>,
//     outputs: BTreeMap<String, ValueKind>,
// }

// impl<Target, Runtime> ModuleBuilder<Target, Runtime> {
//     pub fn target(mut self, target: Target) -> Self {
//         self.target = Some(target);
//         self
//     }

//     pub fn schedule(mut self, schedule: Schedule) -> Self {
//         self.schedule = Some(schedule);
//         self
//     }

//     pub fn runtime(mut self, runtime: Runtime) -> Self {
//         self.runtime = Some(runtime);
//         self
//     }

//     pub fn include_source_code(mut self, name: String, source_code: SourceCode) -> Self {
//         self.source_code.insert(name, source_code);
//         self
//     }

//     pub fn expect_input(mut self, key: String, value_kind: ValueKind) -> Self {
//         self.inputs.insert(key, value_kind);
//         self
//     }

//     pub fn provide_output(mut self, key: String, value_kind: ValueKind) -> Self {
//         self.outputs.insert(key, value_kind);
//         self
//     }

//     pub fn build(self) -> Module {}
// }
// // impl ModuleBuilder<Ta> {
// //     pub fn runtime<R>(&mut self, rt: R)
// //     where
// //         R: Ra,
// //     {
// //     }
// // }

// // impl ModuleBuilder<Tb> {
// //     pub fn runtime<R>(&mut self, rt: R)
// //     where
// //         R: Rb,
// //     {
// //     }
// // }

// pub struct Module<T, S, R> {
//     target: T,
//     schedule: S,
//     runtime: R,
//     source: ModuleSource,
//     inputs: BTreeMap<String, ValueKind>,
//     outputs: BTreeMap<String, ValueKind>,
// }
