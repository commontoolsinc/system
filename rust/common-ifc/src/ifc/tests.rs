use crate::{
    error::ErrorSource,
    ifc::{
        fixtures::{
            Id, Module, TestConfidentiality, TestContext, TestIfcGraph, TestIntegrity, TestLabel,
            TestPolicy,
        },
        IfcGraph,
    },
    CommonIfcError, Result,
};
use common_macros::common_tracing;

use super::fixtures::gen_test_graph;

#[common_tracing]
#[test]
fn it_validates_policy() -> Result<()> {
    let policy = TestPolicy::permissive();
    let input = [(
        "data-1".into(),
        TestLabel::from((TestConfidentiality::High, TestIntegrity::High)),
    )];
    let graph = gen_test_graph();
    graph.validate_graph::<_, TestContext, _>(&policy, &input)?;
    Ok(())
}

#[common_tracing]
#[test]
fn it_rejects_declassified() -> Result<()> {
    let policy = TestPolicy::strict();
    let input = [(
        "data".into(),
        TestLabel::from((TestConfidentiality::High, TestIntegrity::High)),
    )];
    let modules = vec![
        Module::create_root(vec!["data".into()]),
        ("A", vec!["a-in"], vec!["a-out"], TestContext::LowTrust).into(),
    ];
    let edges = vec![((Id::Root, "data".into()), ("A".into(), "a-in".into()))];
    let graph = TestIfcGraph::new(modules, edges);

    let Err(CommonIfcError::ValidationError {
        error_source,
        label_type,
        details: _,
    }) = graph.validate_graph::<_, TestContext, _>(&policy, &input)
    else {
        panic!("Expected ValidationError");
    };
    assert_eq!(
        *error_source,
        ErrorSource {
            source: "ROOT".into(),
            source_port: "data".into(),
            target: "Module_A_".into(),
            target_port: "a-in".into(),
        }
    );
    assert_eq!(&label_type, "Confidentiality");
    Ok(())
}
