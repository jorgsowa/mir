===description===
A standalone function's own `@template T of Base` is bound checked through
its first-class-callable closure the same way a static/instance method's
is — a plain `function(...)` FCC previously built its closure straight from
the function's raw params with no template substitution at all.
===config===
suppress=UnusedVariable
===file===
<?php
class Base {}
class NotBase {}

/**
 * @template T of Base
 * @param T $item
 * @return T
 */
function process($item) {
    return $item;
}

$fn = process(...);
$fn(new NotBase());
===expect===
InvalidArgument@15:4-15:17: Argument $item of {closure}() expects 'Base', got 'NotBase'
