/// Taint analysis helpers.
///
/// A value is "tainted" when it originates from user-controlled input
/// (superglobals, request headers, `@taint-source` calls). Taint is tracked
/// per [`Taint`] kind and propagates through assignments, string operations,
/// array access and pass-through builtins; a sanitizer removes only the kinds
/// it neutralizes.
///
/// When data tainted for a sink's kind reaches it, the matching `Tainted*`
/// issue is emitted.
use php_ast::owned::{Expr, ExprKind, StringPart};

use crate::flow_state::FlowState;

/// Set of sink kinds a value is still dangerous for.
///
/// The low [`KIND_BITS`] hold taint from real sources. While a function body
/// is analyzed for its [`TaintSummary`], each of the first
/// [`MAX_TRACKED_PARAMS`] parameters gets its own slot of the same width, so a
/// sanitizer applied to a parameter is remembered per parameter.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct Taint(u128);

const KIND_BITS: u32 = 14;
const KIND_MASK: u128 = (1 << KIND_BITS) - 1;
pub const MAX_TRACKED_PARAMS: usize = 8;

/// `kinds` copied into the source slot and every parameter slot.
const fn replicate(kinds: u128) -> u128 {
    let kinds = kinds & KIND_MASK;
    let mut out = 0;
    let mut slot = 0;
    while slot <= MAX_TRACKED_PARAMS as u32 {
        out |= kinds << (slot * KIND_BITS);
        slot += 1;
    }
    out
}

impl Taint {
    pub const NONE: Taint = Taint(0);
    pub const HTML: Taint = Taint(1 << 0);
    pub const SQL: Taint = Taint(1 << 1);
    pub const SHELL: Taint = Taint(1 << 2);
    pub const HEADER: Taint = Taint(1 << 3);
    pub const COOKIE: Taint = Taint(1 << 4);
    pub const FILE: Taint = Taint(1 << 5);
    pub const UNSERIALIZE: Taint = Taint(1 << 6);
    pub const EVAL: Taint = Taint(1 << 7);
    pub const LLM_PROMPT: Taint = Taint(1 << 8);
    pub const LDAP: Taint = Taint(1 << 9);
    pub const XPATH: Taint = Taint(1 << 10);
    /// Any `@taint-sink` kind without a dedicated bit.
    pub const OTHER: Taint = Taint(1 << 11);
    pub const SSRF: Taint = Taint(1 << 12);
    pub const CALLABLE: Taint = Taint(1 << 13);
    pub const ALL: Taint = Taint(KIND_MASK);

    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    pub const fn intersects(self, other: Taint) -> bool {
        self.0 & other.0 != 0
    }

    /// Remove `kinds` from the source slot and every parameter slot.
    pub const fn without(self, kinds: Taint) -> Taint {
        Taint(self.0 & !replicate(kinds.0))
    }

    /// Keep only `kinds` in every slot.
    pub const fn restrict_to(self, kinds: Taint) -> Taint {
        Taint(self.0 & replicate(kinds.0))
    }

    /// Add `kinds` to every slot that is already tainted.
    pub fn restore(self, kinds: Taint) -> Taint {
        let mut out = self.0;
        for slot in 0..=MAX_TRACKED_PARAMS as u32 {
            if (self.0 >> (slot * KIND_BITS)) & KIND_MASK != 0 {
                out |= (kinds.0 & KIND_MASK) << (slot * KIND_BITS);
            }
        }
        Taint(out)
    }

    /// Parameter labels only.
    pub const fn labels(self) -> Taint {
        Taint(self.0 & !KIND_MASK)
    }

    /// Taint from real sources only.
    pub const fn concrete(self) -> Taint {
        Taint(self.0 & KIND_MASK)
    }

    /// The label seeded into parameter `index` for summary analysis.
    pub const fn param(index: usize) -> Taint {
        Taint(KIND_MASK << ((index as u32 + 1) * KIND_BITS))
    }

    /// This value without parameter `index`'s label.
    pub const fn without_label(self, index: usize) -> Taint {
        Taint(self.0 & !Taint::param(index).0)
    }

    /// Kinds of parameter `index`'s label this value still carries.
    pub const fn param_kinds(self, index: usize) -> Taint {
        Taint((self.0 >> ((index as u32 + 1) * KIND_BITS)) & KIND_MASK)
    }

    /// `kinds` in label slot `slot`.
    pub const fn label(slot: usize, kinds: Taint) -> Taint {
        Taint((kinds.0 & KIND_MASK) << ((slot as u32 + 1) * KIND_BITS))
    }

    /// Every kind of each label slot this value carries.
    pub fn label_slots(self) -> Taint {
        (0..MAX_TRACKED_PARAMS)
            .filter(|slot| !self.param_kinds(*slot).is_empty())
            .fold(Taint::NONE, |acc, slot| acc | Taint::param(slot))
    }

    pub const fn intersection(self, other: Taint) -> Taint {
        Taint(self.0 & other.0)
    }

    pub const fn difference(self, other: Taint) -> Taint {
        Taint(self.0 & !other.0)
    }
}

impl std::ops::BitOr for Taint {
    type Output = Taint;
    fn bitor(self, rhs: Taint) -> Taint {
        Taint(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for Taint {
    fn bitor_assign(&mut self, rhs: Taint) {
        self.0 |= rhs.0;
    }
}

mod callbacks;
mod escapers;
mod objects;
mod propagation;
mod sinks;
mod sources;
mod summaries;
mod trace;

pub use self::callbacks::*;
pub use self::escapers::*;
pub use self::objects::*;
pub use self::propagation::*;
pub use self::sinks::*;
pub use self::sources::*;
pub use self::summaries::*;
pub use self::trace::*;
