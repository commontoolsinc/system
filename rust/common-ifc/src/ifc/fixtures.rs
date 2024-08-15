use crate::{
    graph::{PortGraph, PortGraphNode},
    ifc::{HasLabelType, IfcContext, IfcLabel, IfcNode, IfcPolicy, LabelType},
    Result,
};
use common_macros::Lattice;
use std::{collections::HashMap, fmt::Display};

#[derive(Lattice, Clone, Hash, Default, Ord, PartialOrd, Eq, PartialEq, Debug)]
pub enum TestIntegrity {
    #[default]
    Low,
    High,
}

impl HasLabelType for TestIntegrity {
    fn label_type() -> LabelType {
        LabelType::Integrity
    }
}

impl Display for TestIntegrity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                TestIntegrity::Low => "LowIntegrity",
                TestIntegrity::High => "HighIntegrity",
            }
        )
    }
}

#[derive(Lattice, Clone, Hash, Default, Ord, PartialOrd, Eq, PartialEq, Debug)]
pub enum TestConfidentiality {
    Low,
    #[default]
    High,
}

impl HasLabelType for TestConfidentiality {
    fn label_type() -> LabelType {
        LabelType::Confidentiality
    }
}

impl Display for TestConfidentiality {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                TestConfidentiality::Low => "LowConfidentiality",
                TestConfidentiality::High => "HighConfidentiality",
            }
        )
    }
}

#[derive(Debug, Clone)]
pub struct TestLabel((TestConfidentiality, TestIntegrity));

impl IfcLabel for TestLabel {
    type Confidentiality = TestConfidentiality;
    type Integrity = TestIntegrity;

    fn confidentiality(&self) -> &Self::Confidentiality {
        &self.0 .0
    }

    fn integrity(&self) -> &Self::Integrity {
        &self.0 .1
    }
}

impl From<(TestConfidentiality, TestIntegrity)> for TestLabel {
    fn from(value: (TestConfidentiality, TestIntegrity)) -> Self {
        Self(value)
    }
}

impl From<TestLabel> for (TestConfidentiality, TestIntegrity) {
    fn from(value: TestLabel) -> Self {
        (value.0 .0, value.0 .1)
    }
}

#[derive(Ord, PartialOrd, Eq, PartialEq, Clone, Debug)]
pub enum TestContext {
    LowTrust,
    HighTrust,
}

impl IfcContext for TestContext {
    type Error = String;
    fn validate(&self, context: &Self) -> ::std::result::Result<(), Self::Error> {
        match self <= context {
            true => Ok(()),
            false => Err("Invalid.".into()),
        }
    }
}

#[derive(Hash, Debug, Eq, PartialEq)]
pub enum Id {
    Name(String),
    Root,
}

impl std::fmt::Display for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Id::Name(s) => write!(f, "Module_{}_", s),
            Id::Root => write!(f, "ROOT"),
        }
    }
}

impl From<&str> for Id {
    fn from(value: &str) -> Self {
        match value {
            "ROOT" => Id::Root,
            _ => Id::Name(value.into()),
        }
    }
}

#[derive(Debug)]
pub struct Module {
    name: Id,
    inputs: Vec<String>,
    outputs: Vec<String>,
    context: Option<TestContext>,
}

impl Module {
    pub fn create_root(ports: Vec<String>) -> Self {
        Self {
            name: Id::Root,
            inputs: ports.clone(),
            outputs: ports,
            context: None,
        }
    }
}

impl IfcNode<TestContext> for Module {
    fn context(&self) -> Option<&TestContext> {
        self.context.as_ref()
    }
}

impl From<(Id, Vec<String>, Vec<String>, TestContext)> for Module {
    fn from(value: (Id, Vec<String>, Vec<String>, TestContext)) -> Self {
        Module {
            name: value.0,
            inputs: value.1,
            outputs: value.2,
            context: Some(value.3),
        }
    }
}

// Shorthand for tests using &str's
impl From<(&str, Vec<&str>, Vec<&str>, TestContext)> for Module {
    fn from(value: (&str, Vec<&str>, Vec<&str>, TestContext)) -> Self {
        Module {
            name: Id::Name(value.0.into()),
            inputs: value.1.iter().map(|s| String::from(*s)).collect(),
            outputs: value.2.iter().map(|s| String::from(*s)).collect(),
            context: Some(value.3),
        }
    }
}

impl PortGraphNode<Id> for Module {
    fn id(&self) -> &Id {
        &self.name
    }
    fn inputs<'a>(&'a self) -> impl Iterator<Item = &'a str> {
        self.inputs.iter().map(|s| s.as_str())
    }
    fn outputs<'a>(&'a self) -> impl Iterator<Item = &'a str> {
        self.outputs.iter().map(|s| s.as_str())
    }
    fn is_root(&self) -> bool {
        matches!(self.name, Id::Root)
    }
}

type Edge = ((Id, String), (Id, String));

#[derive(Debug)]
pub struct TestIfcGraph {
    modules: Vec<Module>,
    edges: Vec<Edge>,
}

impl TestIfcGraph {
    pub fn new(modules: Vec<Module>, edges: Vec<Edge>) -> Self {
        Self { modules, edges }
    }
}

impl PortGraph for TestIfcGraph {
    type NodeId = Id;
    type Node = Module;
    type Edge = Edge;

    fn root(&self) -> Option<&Self::Node> {
        self.modules.iter().next()
    }

    fn nodes(&self) -> impl Iterator<Item = &Self::Node> {
        self.modules.iter()
    }

    fn edges(&self) -> impl Iterator<Item = &Self::Edge> {
        self.edges.iter()
    }

    fn get_node(&self, id: &Self::NodeId) -> Option<&Self::Node> {
        self.modules.iter().find(|node| node.id() == id)
    }
}

pub struct TestPolicy(
    pub  (
        HashMap<TestConfidentiality, TestContext>,
        HashMap<TestIntegrity, TestContext>,
    ),
);

impl TestPolicy {
    pub fn permissive() -> Self {
        Self((
            [
                (TestConfidentiality::Low, TestContext::LowTrust),
                (TestConfidentiality::High, TestContext::LowTrust),
            ]
            .into(),
            [
                (TestIntegrity::Low, TestContext::LowTrust),
                (TestIntegrity::High, TestContext::LowTrust),
            ]
            .into(),
        ))
    }

    pub fn strict() -> Self {
        Self((
            [
                (TestConfidentiality::Low, TestContext::HighTrust),
                (TestConfidentiality::High, TestContext::HighTrust),
            ]
            .into(),
            [
                (TestIntegrity::Low, TestContext::HighTrust),
                (TestIntegrity::High, TestContext::HighTrust),
            ]
            .into(),
        ))
    }
}

impl IfcPolicy for TestPolicy {
    type Label = TestLabel;
    type Context = TestContext;
    fn get_requirements(&self, label: &Self::Label) -> Result<(&Self::Context, &Self::Context)> {
        Ok((
            self.0 .0.get(label.confidentiality()).unwrap(),
            self.0 .1.get(label.integrity()).unwrap(),
        ))
    }
}

/// Constructs a graph with the following structure:
///
///     ROOT
///      ↓ data-1
///      A
///    ↙   ↘
///    ↓   ↓
///    ↓   C
///    ↓ ↙  ↘
///    B      D
///    ↓
///   ROOT (data-1)
///
pub fn gen_test_graph() -> TestIfcGraph {
    let modules = vec![
        Module::create_root(vec!["data-1".into()]),
        ("A", vec!["a-in"], vec!["a-out"], TestContext::LowTrust).into(),
        (
            "B",
            vec!["b-in1", "b-in2"],
            vec!["b-out"],
            TestContext::LowTrust,
        )
            .into(),
        (
            "C",
            vec!["c-in"],
            vec!["c-out1", "c-out2"],
            TestContext::LowTrust,
        )
            .into(),
        ("D", vec!["d-in"], vec![], TestContext::LowTrust).into(),
    ];
    let edges = vec![
        ((Id::Root, "data-1".into()), ("A".into(), "a-in".into())),
        (("A".into(), "a-out".into()), ("B".into(), "b-in1".into())),
        (("A".into(), "a-out".into()), ("C".into(), "c-in".into())),
        (("C".into(), "c-out1".into()), ("B".into(), "b-in2".into())),
        (("C".into(), "c-out2".into()), ("D".into(), "d-in".into())),
        (("B".into(), "b-out".into()), (Id::Root, "data-1".into())),
    ];
    TestIfcGraph::new(modules, edges)
}
