#![cfg(not(target_arch = "wasm32"))]

use anyhow::Result;
use common_protos::{common, formula, runtime::runtime_client::RuntimeClient};
use common_runtime::{
    helpers::{start_runtime, VirtualEnvironment},
    target::formula_vm::{
        AttributeRangeQuery, Datom, Entity, Fact, Instruction, RangeQuery, Scalar, ScalarMap, State,
    },
};
use common_tracing::common_tracing;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[common_tracing]
async fn it_interprets_and_runs_a_common_formula() -> Result<()> {
    let VirtualEnvironment {
        mut runtime_client, ..
    } = start_runtime().await?;

    let source = r#"
export const init = (input) => {
  let state = Object.keys(input).length;
  let key = "float";
  return [state, {
    "ByAttribute": {
      entity: undefined,
      attribute: key,
      value: input[key]
    }
  }];
}

export const step = (total, datoms) => {
  return [total + datoms.length, []]
}

export const end = (total) => [{ "Assert": {
  entity: { id: "foo" },
  attribute: "someattr",
  value: total,
}}]
"#;

    let instance_id = instantiate(&mut runtime_client, source).await?;
    let (state, range_query) = init(
        &mut runtime_client,
        &instance_id,
        vec![
            ("buffer".into(), Vec::<u8>::from([1, 2, 3]).into()),
            ("bool".into(), true.into()),
            ("string".into(), String::from("hello").into()),
            ("integer".into(), 123i32.into()),
            ("float".into(), 3.1415f64.into()),
            ("null".into(), ().into()),
            ("entity".into(), Entity { id: "foo".into() }.into()),
        ]
        .into(),
    )
    .await?;

    assert_eq!(
        range_query,
        RangeQuery::Attribute(AttributeRangeQuery {
            entity: None,
            attribute: "float".into(),
            value: Some(Scalar::Float(3.1415f64)),
        })
    );

    let datoms = vec![];
    let (state, _instructions) = step(&mut runtime_client, &instance_id, state, datoms).await?;
    let instructions = end(&mut runtime_client, &instance_id, state).await?;
    assert_eq!(
        instructions[0],
        Instruction::Assert(Fact {
            entity: Entity { id: "foo".into() },
            attribute: "someattr".into(),
            value: Scalar::Integer(7i32)
        })
    );

    Ok(())
}

async fn instantiate(
    runtime_client: &mut RuntimeClient<tonic::transport::channel::Channel>,
    source: &str,
) -> Result<String> {
    let formula::InstantiateFormulaResponse { instance_id, .. } = runtime_client
        .instantiate_formula(formula::InstantiateFormulaRequest {
            target: common::Target::CommonFormulaVm.into(),
            module_reference: Some(common::ModuleBody {
                variant: Some(common::module_body::Variant::ModuleSource(
                    common::ModuleSource {
                        source_code: [(
                            "module".into(),
                            common::SourceCode {
                                content_type: common::ContentType::JavaScript.into(),
                                body: source.into(),
                            },
                        )]
                        .into(),
                    },
                )),
            }),
        })
        .await?
        .into_inner();
    Ok(instance_id)
}

async fn init(
    runtime_client: &mut RuntimeClient<tonic::transport::channel::Channel>,
    instance_id: &str,
    input: ScalarMap,
) -> Result<(State, RangeQuery)> {
    let formula::RunInitFormulaResponse {
        state, range_query, ..
    } = runtime_client
        .run_init_formula(formula::RunInitFormulaRequest {
            instance_id: instance_id.to_string(),
            input: input.into(),
        })
        .await?
        .into_inner();
    Ok((state, range_query.unwrap().try_into()?))
}

async fn step(
    runtime_client: &mut RuntimeClient<tonic::transport::channel::Channel>,
    instance_id: &str,
    state: State,
    datoms: Vec<Datom>,
) -> Result<(State, Vec<Instruction>)> {
    let formula::RunStepFormulaResponse {
        state,
        instructions,
        ..
    } = runtime_client
        .run_step_formula(formula::RunStepFormulaRequest {
            instance_id: instance_id.to_string(),
            state: state.into(),
            datoms: datoms.into_iter().map(|d| d.into()).collect(),
        })
        .await?
        .into_inner();
    Ok((
        state,
        instructions
            .into_iter()
            .map(|i| i.try_into())
            .collect::<Result<Vec<_>, _>>()?,
    ))
}

async fn end(
    runtime_client: &mut RuntimeClient<tonic::transport::channel::Channel>,
    instance_id: &str,
    state: State,
) -> Result<Vec<Instruction>> {
    let formula::RunEndFormulaResponse { instructions, .. } = runtime_client
        .run_end_formula(formula::RunEndFormulaRequest {
            instance_id: instance_id.to_string(),
            state: state.into(),
        })
        .await?
        .into_inner();
    Ok(instructions
        .into_iter()
        .map(|i| i.try_into())
        .collect::<Result<Vec<_>, _>>()?)
}
