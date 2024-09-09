use crate::{CommonIfcError, Context, Label, Policy, Result};
use common_graph::{Graph, GraphProcessorItem, OwnedGraphData};

/// Evaluates a [`Policy`] against given `inputs` containing
/// [`Label`]s as it propagates through a [`Graph`].
pub fn validate_graph<'ext, I>(
    graph: &Graph<Context>,
    policy: &Policy,
    inputs: I,
) -> Result<OwnedGraphData<Label>>
where
    I: IntoIterator<Item = (&'ext str, Label)>,
{
    let output = graph.process(inputs, move |item: &mut GraphProcessorItem<'_, _, _>| {
        let context = item.node().inner().ok_or_else(|| {
            CommonIfcError::Unexpected("Missing context in graph evaluation.".into())
        })?;

        // All inputs must be written to at this point.
        // We may introduce "optional" inputs in the future,
        // which could be ignored here, or change how
        // ports are referenced in the graph, but for now, verify
        // that claim. Collect instead of passing an iterator
        // to support this.
        let inputs: Vec<_> = item
            .inputs()
            .iter()
            .map(|(k, v)| {
                if let Some(label) = v {
                    Ok((*k, label))
                } else {
                    Err(CommonIfcError::Unexpected(
                        "Context missing label information.".into(),
                    ))
                }
            })
            .collect::<Result<Vec<_>>>()?;

        policy
            .validate(inputs.clone(), context)
            .map_err(|e| match e {
                CommonIfcError::PolicyViolation(mut inner) => {
                    let node = item.node().label().to_string();
                    inner.node = Some(node);
                    inner.into()
                }
                e => e,
            })?;

        let constrained = Label::constrain(inputs.iter().map(|(_, v)| *v));

        for (_, out_value) in item.outputs_mut() {
            **out_value = Some(constrained.clone());
        }

        Ok::<(), CommonIfcError>(())
    })??;
    Ok(output.into_owned())
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crate::{
        error::PolicyViolationSource, validate_graph, CommonIfcError, Confidentiality, Context,
        Integrity, Label, LabelType, ModuleEnvironment, Policy, Result,
    };
    use common_graph::GraphBuilder;

    #[test]
    fn it_validates_policy_graph() -> Result<()> {
        pub use {Confidentiality::*, Integrity::*, ModuleEnvironment::*};

        let srv = Context {
            environment: ModuleEnvironment::Server,
        };
        let brw = Context {
            environment: ModuleEnvironment::WebBrowser,
        };

        let builder = GraphBuilder::default();
        let graph = builder
            .set_label("Storage")
            .set_graph_input(vec![
                "event_reset",
                "event_confirm",
                "todos",
                "api_key",
                "journal",
            ])
            .set_graph_output(vec![
                "event_reset",
                "event_confirm",
                "todos",
                "api_key",
                "journal",
            ])
            .node(
                "PromptUI",
                brw.clone(),
                vec!["todos", "journal", "reset_trigger", "confirm_trigger"],
                vec!["prompt", "rendertree"],
            )
            .node("LLM", srv.clone(), vec!["prompt", "api_key"], vec!["dream"])
            .node("ConfirmUI", brw.clone(), vec!["dream"], vec!["rendertree"])
            .node(
                "RenderSink",
                brw.clone(),
                vec!["rendertree1", "rendertree2"],
                vec!["event_reset", "event_confirm"],
            )
            .connect_input("event_reset", ("PromptUI", "reset_trigger"))?
            .connect_input("event_confirm", ("PromptUI", "confirm_trigger"))?
            .connect_input("todos", ("PromptUI", "todos"))?
            .connect_input("journal", ("PromptUI", "journal"))?
            .connect_input("api_key", ("LLM", "api_key"))?
            .connect(("PromptUI", "prompt"), ("LLM", "prompt"))?
            .connect(("LLM", "dream"), ("ConfirmUI", "dream"))?
            .connect(("PromptUI", "rendertree"), ("RenderSink", "rendertree1"))?
            .connect(("ConfirmUI", "rendertree"), ("RenderSink", "rendertree2"))?
            .connect_output(("RenderSink", "event_reset"), "event_reset")?
            .connect_output(("RenderSink", "event_confirm"), "event_confirm")?
            .build()?;

        let not_secret = Label::from((Confidentiality::Public, Integrity::Low));
        let secret = Label::from((Confidentiality::Private, Integrity::Low));

        let inputs = [
            ("event_reset", not_secret.clone()),
            ("event_confirm", not_secret.clone()),
            ("todos", secret.clone()),
            ("api_key", secret.clone()),
            ("journal", secret.clone()),
        ];

        let policy = Policy::new(
            BTreeMap::from([(Public, (Server,).into()), (Private, (Server,).into())]),
            BTreeMap::from([(Low, (Server,).into()), (High, (Server,).into())]),
        )?;
        let strict_policy = Policy::new(
            BTreeMap::from([(Public, (Server,).into()), (Private, (WebBrowser,).into())]),
            BTreeMap::from([(Low, (Server,).into()), (High, (Server,).into())]),
        )?;

        validate_graph(&graph, &policy, inputs.clone())?;

        let Err(e) = validate_graph(&graph, &strict_policy, inputs.clone()) else {
            panic!("Expected graph invalidation.");
        };
        assert_eq!(
            e,
            CommonIfcError::from(PolicyViolationSource {
                cause: CommonIfcError::InvalidEnvironment,
                input: String::from("prompt"),
                label_type: LabelType::Confidentiality,
                node: Some(String::from("LLM")),
            })
        );
        Ok(())
    }
}
