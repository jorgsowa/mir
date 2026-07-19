===description===
`inherited_template_bindings`'s `apply_type_args` must resolve a bare
pass-through middle class's (`MidBox extends BaseBox {}`, no own `@template`)
inherited `@template` via `class_template_params`, not the own-declarations-
only query — otherwise `@extends MidBox<int>` on `IntBox` never binds `T`
and `get()`'s inherited `@return T` leaks through unresolved instead of
narrowing to `int`. The receiver comes from a plain parameter (not `new
IntBox(...)`) so the binding can only come from this ancestor-chain
resolution, not constructor-arg inference, and `get()`'s body throws instead
of returning a literal so the type can't be inferred from the body either.
===config===
suppress=UnusedVariable,MissingThrowsDocblock
===file===
<?php
/** @template T */
class BaseBox {
    /** @return T */
    public function get() {
        throw new \Exception();
    }
}

class MidBox extends BaseBox {}

/** @extends MidBox<int> */
class IntBox extends MidBox {}

function test(IntBox $box): void {
    $x = $box->get();
    /** @mir-check $x is int */
    echo "ok";
}
===expect===
