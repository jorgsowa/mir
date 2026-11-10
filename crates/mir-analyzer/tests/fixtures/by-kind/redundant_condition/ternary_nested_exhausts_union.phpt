===description===
A nested ternary re-testing the only member left of a union is always true; each ternary of a concatenation is reported on its own condition.
===file===
<?php
final class Circle {}
final class Line {}

function nested(Circle|Line $shape): string {
    return $shape instanceof Circle ? 'circle' : ($shape instanceof Line ? 'line' : 'none');
//                                                ^^^^^^^^^^^^^^^^^^^^^^ RedundantCondition: Condition is always true, so the else branch is never reached
}

function concatenated(int $n): string {
    return (is_int($n) ? 'int' : 'other') . (is_string($n) ? 'string' : 'other');
//          ^^^^^^^^^^ RedundantCondition: Condition is always true, so the else branch is never reached
//                                           ^^^^^^^^^^^^^ RedundantCondition: Condition is always false, so the then branch is never reached
}
