use crate::{graph::PortGraph, ifc::validation::validate_graph, CommonIfcError, Result};
use std::{
    fmt::{Debug, Display},
    slice::Iter,
};

/// A trait representing a partially ordered lattice.
/// Derive [common_macros::Lattice] to implement.
pub trait Lattice: Display + Debug + Ord + PartialOrd + Eq + PartialEq + Sized {
    /// Return the top-most principal for this lattice.
    fn top() -> Self;
    /// Return the bottom-most principal for this lattice.
    fn bottom() -> Self;
    /// Returns an iterator that iterates over
    /// all variants, from lowest to highest.
    fn iter() -> Iter<'static, Self>;
}

#[derive(PartialEq, Debug, Clone)]
/// Type to distinguish between confidentiality and integrity
/// labels in debugging.
pub enum LabelType {
    /// Represents a confidentiality label.
    Confidentiality,
    /// Represents an integrity label.
    Integrity,
}

/// A trait to distinguish between confidentiality
/// and integrity label types.
pub trait HasLabelType {
    /// Returns a [LabelType] for this type.
    fn label_type() -> LabelType;
}

impl Display for LabelType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                LabelType::Confidentiality => "Confidentiality",
                LabelType::Integrity => "Integrity",
            }
        )
    }
}

/// Trait for implementing confidentiality and integrity
/// levels.
pub trait IfcLabel:
    From<(Self::Confidentiality, Self::Integrity)>
    + Into<(Self::Confidentiality, Self::Integrity)>
    + Debug
    + Clone
{
    /// Type of confidentiality label.
    type Confidentiality: Lattice + HasLabelType + Clone;
    /// Type of integrity label.
    type Integrity: Lattice + HasLabelType + Clone;
    /// Return the confidentiality label.
    fn confidentiality(&self) -> &Self::Confidentiality;
    /// Return the integrity label.
    fn integrity(&self) -> &Self::Integrity;

    /// Creates a new [IfcLabel] that sets its [IfcLabel::Confidentiality]
    /// and [IfcLabel::Integrity] to the highest confidentiality
    /// and lowest integrity found in `input`.
    fn constrain<'a, L, I>(input: I) -> L
    where
        L: IfcLabel + 'a,
        I: IntoIterator<Item = (&'a String, &'a L)>,
    {
        let mut max_conf = &L::Confidentiality::bottom();
        let mut min_int = &L::Integrity::top();
        for (_, label) in input {
            max_conf = std::cmp::max(max_conf, label.confidentiality());
            min_int = std::cmp::min(min_int, label.integrity());
        }
        L::from((max_conf.to_owned(), min_int.to_owned()))
    }
}

/// Trait implementing policies, a mapping of an [IfcLabel]
/// to its [IfcContext] requirements. All labels must be
/// represented by some context.
pub trait IfcPolicy {
    /// Label type.
    type Label: IfcLabel;
    /// Context type.
    type Context: IfcContext;

    /// Return the confidentiality and integrity requirements
    /// for the provided `label`.
    fn get_requirements(&self, label: &Self::Label) -> Result<(&Self::Context, &Self::Context)>;

    /// Checks if the provided `context` is allowed by the requirements
    /// mapped to by `label`.
    fn check_context(&self, label: &Self::Label, context: &Self::Context) -> Result<()> {
        let (conf_reqs, int_reqs) = self.get_requirements(label)?;
        match (conf_reqs.validate(context), int_reqs.validate(context)) {
            (Err(e), _) => Err(CommonIfcError::InvalidContext {
                label_type: LabelType::Confidentiality,
                details: e.to_string(),
            }),
            (_, Err(e)) => Err(CommonIfcError::InvalidContext {
                label_type: LabelType::Integrity,
                details: e.to_string(),
            }),
            (Ok(_), Ok(_)) => Ok(()),
        }
    }
}

/// A node in a [IfcGraph] that contains a [IfcContext].
pub trait IfcNode<Context: IfcContext> {
    /// Returns the [IfcContext] associated with this node.
    fn context(&self) -> Option<&Context>;
}

/// Trait representing a context that is used as policy requirements,
/// validating against a context in an execution node.
pub trait IfcContext {
    /// Error type of validation.
    type Error: Display;
    /// Validate that the provided `context` meets
    /// this context's criteria.
    fn validate(&self, context: &Self) -> ::std::result::Result<(), Self::Error>;
}

/// Trait for validating an IFC graph. Must implement
/// [PortGraph] with [IfcNode] node types.
pub trait IfcGraph<Context: IfcContext>
where
    Self: PortGraph,
    <Self as PortGraph>::Node: IfcNode<Context>,
{
    /// Validate graph against input and policy.
    fn validate_graph<'a, I, C, P>(&'a self, policy: &P, input: I) -> Result<()>
    where
        Self: Sized + PortGraph,
        P: IfcPolicy<Context = Context>,
        I: IntoIterator<Item = &'a (String, P::Label)>,
        P::Label: 'a,
    {
        validate_graph(self, policy, input)
    }
}

impl<Context, T> IfcGraph<Context> for T
where
    T: PortGraph,
    Context: IfcContext,
    <Self as PortGraph>::Node: IfcNode<Context>,
{
}
