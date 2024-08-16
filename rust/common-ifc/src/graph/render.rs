use crate::{
    graph::{PortGraph, PortGraphEdge, PortGraphNode, PortType},
    CommonIfcError, Result,
};
use std::io::Write;

/// Render options for the [render_with_options] method.
#[derive(Clone)]
pub struct RenderOpts {
    /// Graph name to display when rendering.
    pub graph_name: String,
    /// Style for invisible edges. By default, they're
    /// invisible, but may be customized for debugging purposes.
    pub invisible_style: String,
    /// Font string used in render output.
    pub font_name: String,
    /// Font size used in render output.
    pub font_size: u16,
    /// Background color of node objects.
    pub node_background_color: String,
    /// Background color of port objects.
    pub port_background_color: String,
    /// Background color of port groups.
    pub port_group_background_color: String,
}

impl Default for RenderOpts {
    fn default() -> Self {
        RenderOpts {
            graph_name: "".into(),
            invisible_style: "[style=invis]".into(),
            font_name: "Helvetica,Arial,sans-serif".into(),
            font_size: 12,
            node_background_color: "darkgrey".into(),
            port_background_color: "white".into(),
            port_group_background_color: "lightgrey".into(),
        }
    }
}

/// Builder pattern for creating a [RenderOpts] via
/// [RenderOptsBuilder::build], or rendering directly via
/// [RenderOptsBuilder::render].
#[derive(Default)]
pub struct RenderOptsBuilder {
    opts: RenderOpts,
}

impl RenderOptsBuilder {
    /// Set [RenderOpts]' `graph_name`.
    pub fn graph_name(mut self, graph_name: &str) -> Self {
        self.opts.graph_name = graph_name.into();
        self
    }

    /// Set [RenderOpts]' `invisible_style`.
    pub fn invisible_style(mut self, style: &str) -> Self {
        self.opts.invisible_style = style.into();
        self
    }

    /// Sets [RenderOpts]' `invisible_style` to
    /// a filled, red style for debugging.
    pub fn render_invisibles(mut self) -> Self {
        self.opts.invisible_style = "[style=filled,color=red]".into();
        self
    }

    /// Set [RenderOpts]' `font_name`.
    pub fn font_name(mut self, font_name: &str) -> Self {
        self.opts.font_name = font_name.into();
        self
    }

    /// Set [RenderOpts]' `font_size`.
    pub fn font_size(mut self, font_size: u16) -> Self {
        self.opts.font_size = font_size;
        self
    }

    /// Set [RenderOpts]' `node_background_color`.
    pub fn node_background_color(mut self, color: &str) -> Self {
        self.opts.node_background_color = color.into();
        self
    }

    /// Set [RenderOpts]' `port_background_color`.
    pub fn port_background_color(mut self, color: &str) -> Self {
        self.opts.port_background_color = color.into();
        self
    }

    /// Set [RenderOpts]' `port_group_background_color`.
    pub fn port_group_background_color(mut self, color: &str) -> Self {
        self.opts.port_group_background_color = color.into();
        self
    }

    /// Return the built [RenderOpts] from this builder.
    pub fn build(self) -> RenderOpts {
        self.opts
    }

    /// Call [render_with_options] with the [RenderOpts] from this builder.
    pub fn render<G: PortGraph, W: Write>(self, graph: &G, w: W) -> Result<()> {
        render_with_options(graph, w, &self.opts)
    }
}

/// Render [PortGraph] as a DOT file to provided writer
/// with default rendering options.
/// See [render_with_options] for more.
pub fn render<G: PortGraph, W: Write>(graph: &G, w: W) -> Result<()> {
    render_with_options(graph, w, &RenderOpts::default())
}

/// Generates output in [Graphviz](https://www.graphviz.org/) [DOT](https://www.graphviz.org/doc/info/lang.html) format.
///
/// Traverses the provided [PortGraph] and writes a DOT file into the provided
/// writer.
pub fn render_with_options<G: PortGraph, W: Write>(
    graph: &G,
    mut w: W,
    options: &RenderOpts,
) -> Result<()> {
    let graph_name = &options.graph_name;
    let port_bg = &options.port_background_color;
    let font_name = &options.font_name;
    let font_size = &options.font_size;
    writeln!(
        w,
        r#"digraph {{
label = "{graph_name}";
graph [fontsize={font_size} fontname="{font_name}"];
node [style="rounded,filled" color={port_bg} shape=rectangle fontsize={font_size} fontname="{font_name}"];
edge [fontsize={font_size} fontname="{font_name}"];
rankdir = "TB";
"#
    )?;

    for node in graph.nodes() {
        render_node::<G, W>(&mut w, node, options)?;
        render_outgoing_edges(&mut w, graph, node)?;
    }

    writeln!(w, "}}")?;

    Ok(())
}

/// Render a [PortGraph::Node] and its ports as a `subgraph`.
fn render_node<G: PortGraph, W: Write>(
    w: &mut W,
    node: &G::Node,
    options: &RenderOpts,
) -> Result<()> {
    let node_id = node.id();
    validate_slug(&node_id.to_string())?;

    writeln!(
        w,
        r#"
subgraph {node_id} {{
  label = "{node_id}";
  style = "rounded,filled";
  color = {};
  cluster = true;
"#,
        options.node_background_color
    )?;

    let input_slugs = render_ports::<G, W>(w, node, &options, PortType::Input)?;
    let output_slugs = render_ports::<G, W>(w, node, &options, PortType::Output)?;

    // Render invisible connections between an input
    // and an output port to create a hierarchy.
    if let (Some(input_slug), Some(output_slug)) = (input_slugs.first(), output_slugs.first()) {
        writeln!(
            w,
            r#"  {} -> {} {};"#,
            input_slug, output_slug, options.invisible_style
        )?;
    }

    writeln!(w, "}}")?;

    Ok(())
}

/// Renders [PortGraph::Edge] where the edge source
/// is an output port from `node`.
fn render_outgoing_edges<G: PortGraph, W: Write>(
    w: &mut W,
    graph: &G,
    node: &G::Node,
) -> Result<()> {
    let node_id = node.id();
    let connections = graph.get_connections(node, Some(PortType::Output));
    for connection in connections {
        let source = connection.source();
        if source.0 == node_id {
            let target = connection.target();
            writeln!(
                w,
                r#""{}" -> "{}";"#,
                slugify_port::<G>(source.0, source.1, &PortType::Output)?,
                slugify_port::<G>(target.0, target.1, &PortType::Input)?
            )?;
        }
    }
    Ok(())
}

/// Renders a group of `node`'s ports as a `subgraph`.
fn render_ports<G: PortGraph, W: Write>(
    w: &mut W,
    node: &G::Node,
    options: &RenderOpts,
    port_type: PortType,
) -> Result<Vec<String>> {
    let node_id = node.id();
    let (ports, rank) = match port_type {
        PortType::Input => (node.inputs().collect::<Vec<_>>(), "source"),
        PortType::Output => (node.outputs().collect::<Vec<_>>(), "same"),
    };

    // Open anonymous subgraph,
    // set rank for inputs to order above outputs,
    // and override `label`, otherwise repeats the Node label.
    writeln!(
        w,
        r#"
  subgraph {{
    rank = {};
    label = "{}";
    color = "{}";
"#,
        rank,
        port_type.as_str(),
        options.port_group_background_color
    )?;

    let mut port_slugs = vec![];
    for port in ports {
        let port_slug = slugify_port::<G>(node_id, port, &port_type)?;
        writeln!(w, r#"    "{}" [label="{}"];"#, port_slug, port)?;
        port_slugs.push(format!(r#""{}""#, port_slug));
    }
    // Close anonymous subgraph
    writeln!(w, "  }}")?;

    Ok(port_slugs)
}

fn slugify_port<G: PortGraph>(
    node_id: &G::NodeId,
    port: &G::PortName,
    port_type: &PortType,
) -> Result<String> {
    let slug = format!("{}__{}__{}", node_id, port_type.as_str(), port);
    validate_slug(&slug)?;
    Ok(slug)
}

/// Validates a generated slug for compatibility with dot.
/// The first character must be an alphanumeric or '_',
/// and all remaining characters must be alphanumeric or '_'.
fn validate_slug(slug: &str) -> Result<()> {
    match slug.chars().next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' => {}
        _ => {
            return Err(CommonIfcError::InternalError(format!(
                "Invalid slug: {}",
                slug
            )))
        }
    }
    if !slug.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        return Err(CommonIfcError::InternalError(format!(
            "Invalid slug: {}",
            slug
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{
        graph::{RenderOptsBuilder, TestGraph},
        Result,
    };

    #[test]
    fn it_renders_to_dot_format() -> Result<()> {
        let graph = TestGraph::from_iters(
            [
                ("Gems", vec![], vec!["todos", "api_key", "journal"]),
                (
                    "Events",
                    vec!["event_foo_reset", "event_foo_confirm"],
                    vec!["event_foo_reset", "event_foo_confirm"],
                ),
                (
                    "PromptUI",
                    vec!["todos", "journal", "reset_trigger", "confirm_trigger"],
                    vec!["prompt", "rendertree"],
                ),
                ("ConfirmUI", vec!["dream"], vec!["rendertree"]),
                ("LLM", vec!["prompt", "api_key"], vec!["dream"]),
                (
                    "RenderSink",
                    vec!["rendertree"],
                    vec!["event_foo_reset", "event_foo_confirm"],
                ),
            ],
            [
                (("Events", "event_foo_reset"), ("PromptUI", "reset_trigger")),
                (
                    ("Events", "event_foo_confirm"),
                    ("PromptUI", "confirm_trigger"),
                ),
                (("Gems", "todos"), ("PromptUI", "todos")),
                (("Gems", "journal"), ("PromptUI", "journal")),
                (("PromptUI", "prompt"), ("LLM", "prompt")),
                (("Gems", "api_key"), ("LLM", "api_key")),
                (("LLM", "dream"), ("ConfirmUI", "dream")),
                (("PromptUI", "rendertree"), ("RenderSink", "rendertree")),
                (("ConfirmUI", "rendertree"), ("RenderSink", "rendertree")),
                (
                    ("RenderSink", "event_foo_reset"),
                    ("Events", "event_foo_reset"),
                ),
                (
                    ("RenderSink", "event_foo_confirm"),
                    ("Events", "event_foo_confirm"),
                ),
            ],
        );

        let mut out = std::io::Cursor::new(vec![]);
        RenderOptsBuilder::default()
            .graph_name("IFC Example")
            .render(&graph, &mut out)?;
        let dot = String::from_utf8(out.into_inner())?;
        assert!(dot.contains("subgraph Gems"));
        Ok(())
    }
}
