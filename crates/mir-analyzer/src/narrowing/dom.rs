//! `$node->nodeType === XML_ELEMENT_NODE` narrows a `DOMNode` variable to the node class
//! PHP creates for that type (`DOMElement`).
use php_ast::owned::{Expr, ExprKind};

use mir_types::{Atomic, Type};

use crate::flow_state::FlowState;

use super::core::extract_var_name;

pub(super) fn narrow_dom_node_type(ctx: &mut FlowState, left: &Expr, right: &Expr, is_true: bool) {
    if !is_true {
        return;
    }
    let (fetch, class) = match (node_type_class(right), node_type_class(left)) {
        (Some(class), _) => (left, class),
        (None, Some(class)) => (right, class),
        (None, None) => return,
    };
    let ExprKind::PropertyAccess(pa) = &fetch.kind else {
        return;
    };
    let ExprKind::Identifier(prop) = &pa.property.kind else {
        return;
    };
    if &**prop != "nodeType" {
        return;
    }
    let Some(var) = extract_var_name(&pa.object) else {
        return;
    };
    let current = ctx.get_var(&var);
    if !current.types.iter().any(is_dom_node) {
        return;
    }
    let mut narrowed = Type::empty();
    for atomic in &current.types {
        if is_dom_node(atomic) {
            narrowed.add_type(Atomic::TNamedObject {
                fqcn: class.into(),
                type_params: mir_types::union::empty_type_params(),
            });
        } else if !matches!(atomic, Atomic::TNull) {
            // `null->nodeType` is null, which matches no node type.
            narrowed.add_type(atomic.clone());
        }
    }
    ctx.narrow_var(&var, narrowed);
}

fn is_dom_node(atomic: &Atomic) -> bool {
    matches!(atomic, Atomic::TNamedObject { fqcn, .. } if fqcn.eq_ignore_ascii_case("DOMNode"))
}

/// The class of the nodes whose `nodeType` is the `XML_*_NODE` constant or value `expr`.
fn node_type_class(expr: &Expr) -> Option<&'static str> {
    let value = match &expr.kind {
        ExprKind::Int(n) => *n,
        ExprKind::Name(name) if name.part_count() == 1 => match name.as_str() {
            "XML_ELEMENT_NODE" => 1,
            "XML_ATTRIBUTE_NODE" => 2,
            "XML_TEXT_NODE" => 3,
            "XML_CDATA_SECTION_NODE" => 4,
            "XML_ENTITY_REF_NODE" => 5,
            "XML_PI_NODE" => 7,
            "XML_COMMENT_NODE" => 8,
            "XML_DOCUMENT_NODE" => 9,
            "XML_DOCUMENT_TYPE_NODE" => 10,
            "XML_DOCUMENT_FRAG_NODE" => 11,
            "XML_NOTATION_NODE" => 12,
            _ => return None,
        },
        _ => return None,
    };
    Some(match value {
        1 => "DOMElement",
        2 => "DOMAttr",
        3 => "DOMText",
        4 => "DOMCdataSection",
        5 => "DOMEntityReference",
        7 => "DOMProcessingInstruction",
        8 => "DOMComment",
        9 => "DOMDocument",
        10 => "DOMDocumentType",
        11 => "DOMDocumentFragment",
        12 => "DOMNotation",
        _ => return None,
    })
}
