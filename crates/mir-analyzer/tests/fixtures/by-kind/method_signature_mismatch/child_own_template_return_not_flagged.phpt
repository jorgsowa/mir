===description===
FALSE POSITIVE: a child override declaring its own, differently-named,
BOUNDED method-level @template for the return type must not be compared
against the class's concretely-bound ancestor template via
scalar_return_types_compatible. check_overrides' return-type covariance
check substituted the class's inherited bindings and re-checked whether the
PARENT side still had a lingering template before giving up, but never
re-checked the CHILD side — so the still-unbound child template (never
touched by inherited_template_bindings, which only knows class-level
bindings) was compared as if it were a concrete type incompatible with the
ancestor's. (An unbounded `@template U` happens to dodge this particular
path because a bound-less template's `is_mixed()` is true, tripping the
adjacent mixed-return early-out instead — a bound is needed to reach the
buggy comparison.)
===config===
suppress=UnusedParam
===file===
<?php
/** @template T */
interface Repo {
    /** @return T */
    public function find();
}

/** @implements Repo<int> */
class IntRepo implements Repo {
    /**
     * @template U of Countable
     * @return U
     */
    public function find() {
        return 1;
    }
}
===expect===
