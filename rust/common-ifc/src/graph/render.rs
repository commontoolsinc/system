use crate::{
    graph::{PortGraph, PortGraphEdge, PortGraphNode, PortType},
    Result,
};
use std::io::Write;

/// Render options for the [render_with_options] method.
pub struct RenderOpts {
    /// Graph name to display when rendering.
    pub graph_name: String,
    /// Whether invisible edges for structural purposes
    /// should be visible for debugging purposes.
    pub render_invisibles: bool,
    /// Font string used in render output.
    pub font_name: String,
    /// Font size used in render output.
    pub font_size: u16,
    /// Background color of node objects.
    pub node_background_color: String,
    /// Background color of port objects.
    pub port_background_color: String,
}

impl Default for RenderOpts {
    fn default() -> Self {
        RenderOpts {
            graph_name: "".into(),
            render_invisibles: false,
            font_name: "Helvetica,Arial,sans-serif".into(),
            font_size: 12,
            node_background_color: "lightgrey".into(),
            port_background_color: "white".into(),
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

    /// Enable [RenderOpts]' `render_invisibles`.
    pub fn render_invisibles(mut self) -> Self {
        self.opts.render_invisibles = true;
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

    /// Return the built [RenderOpts] from this builder.
    pub fn build(self) -> RenderOpts {
        self.opts
    }

    /// Call [render_with_options] with the [RenderOpts] from this builder.
    pub fn render<G: PortGraph, W: Write>(self, graph: &G, w: W) -> Result<()> {
        render_with_options(graph, w, self.opts)
    }
}

/// Render [PortGraph] as a DOT file to provided writer
/// with default rendering options.
/// See [render_with_options] for more.
pub fn render<G: PortGraph, W: Write>(graph: &G, w: W) -> Result<()> {
    render_with_options(graph, w, RenderOpts::default())
}

/// Generates output in [Graphviz](https://www.graphviz.org/) [DOT](https://www.graphviz.org/doc/info/lang.html) format.
///
/// Traverses the provided [PortGraph] and writes a DOT file into the provided
/// writer.
///
/// Issues:
/// * Need to determine what 'dot' legal names are to properly
///   slugify nodes and ports.
/// * Need to uniquely slug inputs vs outputs as they can have the same name
/// * Port names are "scoped" by node names, though there could be collision
///   e.g. "ModuleX_" and port "foo" could collide with "Module" and port "X_foo"
///   based on concat/slugging.
pub fn render_with_options<G: PortGraph, W: Write>(
    graph: &G,
    mut w: W,
    options: RenderOpts,
) -> Result<()> {
    let graph_name = options.graph_name;
    let port_bg = options.port_background_color;
    let node_bg = options.node_background_color;
    let font_name = options.font_name;
    let font_size = options.font_size;
    let invis_style = match options.render_invisibles {
        true => "[style=filled,color=red]",
        false => "[style=invis]",
    };

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
        let node_id = node.id();
        writeln!(
            w,
            r#"
subgraph {node_id} {{
  label = "{node_id}";
  style = "rounded,filled";
  color = {node_bg};
  cluster = true;
"#
        )?;
        let input_slugs = render_ports::<G, W>(&mut w, node, PortType::Input)?;
        let output_slugs = render_ports::<G, W>(&mut w, node, PortType::Output)?;

        // Render invisible connections between an input
        // and an output port to create a hierarchy.
        if let (Some(input_slug), Some(output_slug)) = (input_slugs.first(), output_slugs.first()) {
            writeln!(w, r#"  {} -> {} {};"#, input_slug, output_slug, invis_style)?;
        }

        writeln!(w, "}}")?;
    }

    for node in graph.nodes() {
        let node_id = node.id();
        let connections = graph.get_connections(node, Some(PortType::Output));
        for connection in connections {
            let source = connection.source();
            if source.0 == node_id {
                let target = connection.target();
                writeln!(
                    w,
                    r#"  "{}" -> "{}";"#,
                    slugify_port::<G>(source.0, source.1),
                    slugify_port::<G>(target.0, target.1)
                )?;
            }
        }
    }

    writeln!(w, "}}")?;

    fn render_ports<G: PortGraph, W: Write>(
        w: &mut W,
        node: &G::Node,
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
    rank = {rank};
    label = "";
"#
        )?;

        let mut port_slugs = vec![];
        for port in ports {
            let port_slug = slugify_port::<G>(node_id, port);
            writeln!(w, r#"    "{}" [label="{}"];"#, port_slug, port)?;
            port_slugs.push(format!(r#""{}""#, port_slug));
        }
        // Close anonymous subgraph
        writeln!(w, "  }}")?;

        Ok(port_slugs)
    }

    fn slugify_port<G: PortGraph>(node_id: &G::NodeId, port: &G::PortName) -> String {
        format!("{}__{}", node_id, port)
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
                (
                    "Gems",
                    vec!["default_in"],
                    vec!["prompt", "openai_key", "anthropic_key"],
                ),
                ("ChatGPT", vec!["prompt", "api_key"], vec!["dream"]),
                ("Claude", vec!["prompt", "api_key"], vec!["dream"]),
                ("Synthesize", vec!["result1", "result2"], vec!["final_out"]),
            ],
            [
                (("Gems", "prompt"), ("ChatGPT", "prompt")),
                (("Gems", "openai_key"), ("ChatGPT", "api_key")),
                (("Gems", "prompt"), ("Claude", "prompt")),
                (("Gems", "anthropic_key"), ("Claude", "api_key")),
                (("ChatGPT", "dream"), ("Synthesize", "result1")),
                (("Claude", "dream"), ("Synthesize", "result2")),
                (("Synthesize", "final_out"), ("Gems", "default_in")),
            ],
        );

        let mut buffer = std::io::Cursor::new(vec![]);
        RenderOptsBuilder::default()
            .graph_name("IFC Example")
            .render(&graph, &mut buffer)?;
        let out = String::from_utf8(buffer.into_inner())?;
        assert!(out.contains("subgraph Gems"));
        Ok(())
    }
}
