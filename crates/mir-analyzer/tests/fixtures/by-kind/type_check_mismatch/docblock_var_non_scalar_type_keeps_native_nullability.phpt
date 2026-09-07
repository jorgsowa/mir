===description===
FP-L28: a `@var` docblock type for a non-scalar property that omits `|null`
must not drop the native hint's own nullability — PHP enforces `?Foo`
regardless of what the docblock says. Property analogue of the fixed
param-side E2 (`docblock_param_non_scalar_type_keeps_native_nullability`).
===config===
suppress=MissingConstructor,UnusedVariable
===file===
<?php
class Foo {}

class Box {
    /** @var Foo */
    public ?Foo $item = null;
}

function check(Box $b): void {
    /** @mir-check $b->item is Foo|null */
    $x = $b->item;
}
===expect===
