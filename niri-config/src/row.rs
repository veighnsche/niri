use knuffel::errors::DecodeError;

use crate::LayoutPart;

// TEAM_055: Renamed from Workspace to RowConfig
#[derive(knuffel::Decode, Debug, Clone, PartialEq)]
pub struct RowConfig {
    #[knuffel(argument)]
    pub name: RowName,
    #[knuffel(child, unwrap(argument))]
    pub open_on_output: Option<String>,
    #[knuffel(child)]
    pub layout: Option<RowLayoutPart>,
}

// TEAM_055: Renamed from WorkspaceName to RowName
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RowName(pub String);

// TEAM_055: Renamed from WorkspaceLayoutPart to RowLayoutPart
#[derive(Debug, Clone, PartialEq)]
pub struct RowLayoutPart(pub LayoutPart);

impl<S: knuffel::traits::ErrorSpan> knuffel::Decode<S> for RowLayoutPart {
    fn decode_node(
        node: &knuffel::ast::SpannedNode<S>,
        ctx: &mut knuffel::decode::Context<S>,
    ) -> Result<Self, DecodeError<S>> {
        for child in node.children() {
            let name = &**child.node_name;

            // Check for disallowed properties.
            //
            // TEAM_055: Updated terminology from workspace to row
            // - empty-row-above-first is a monitor-level concept.
            // - insert-hint customization could make sense for rows, however currently it is
            //   also handled at the monitor level (since insert hints in-between rows are a
            //   monitor-level concept), so for now this config option would do nothing.
            if matches!(name, "empty-row-above-first" | "insert-hint") {
                ctx.emit_error(DecodeError::unexpected(
                    child,
                    "node",
                    format!("node `{name}` is not allowed inside `row.layout`"),
                ));
            }
        }

        LayoutPart::decode_node(node, ctx).map(Self)
    }
}

impl<S: knuffel::traits::ErrorSpan> knuffel::DecodeScalar<S> for RowName {
    fn type_check(
        type_name: &Option<knuffel::span::Spanned<knuffel::ast::TypeName, S>>,
        ctx: &mut knuffel::decode::Context<S>,
    ) {
        if let Some(type_name) = &type_name {
            ctx.emit_error(DecodeError::unexpected(
                type_name,
                "type name",
                "no type name expected for this node",
            ));
        }
    }

    fn raw_decode(
        val: &knuffel::span::Spanned<knuffel::ast::Literal, S>,
        ctx: &mut knuffel::decode::Context<S>,
    ) -> Result<RowName, DecodeError<S>> {
        // TEAM_055: Renamed from WorkspaceNameSet to RowNameSet
        #[derive(Debug)]
        struct RowNameSet(Vec<String>);
        match &**val {
            knuffel::ast::Literal::String(ref s) => {
                let mut name_set: Vec<String> = match ctx.get::<RowNameSet>() {
                    Some(h) => h.0.clone(),
                    None => Vec::new(),
                };

                if name_set.iter().any(|name| name.eq_ignore_ascii_case(s)) {
                    ctx.emit_error(DecodeError::unexpected(
                        val,
                        "named row",
                        format!("duplicate named row: {s}"),
                    ));
                    return Ok(Self(String::new()));
                }

                name_set.push(s.to_string());
                ctx.set(RowNameSet(name_set));
                Ok(Self(s.clone().into()))
            }
            _ => {
                ctx.emit_error(DecodeError::unsupported(
                    val,
                    "row names must be strings",
                ));
                Ok(Self(String::new()))
            }
        }
    }
}
