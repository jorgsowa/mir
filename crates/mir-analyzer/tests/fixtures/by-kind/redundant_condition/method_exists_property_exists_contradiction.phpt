===description===
`method_exists($x, ...)` / `property_exists($x, ...)` true-branch on a
receiver that's neither object-like, string-like, mixed, nor scalar
(e.g. plain `int`) is a provable contradiction (PHP 8 throws a
TypeError for such an argument) — this branch is unreachable, mirroring
every sibling `is_*()` narrower instead of silently reverting to the
unnarrowed type.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
function methodExistsUnreachable(int $x): void {
    if (method_exists($x, 'foo')) {
//      ^^^^^^^^^^^^^^^^^^^^^^^^ RedundantCondition: Condition of type 'bool' always evaluates the same way, so one branch is unreachable
//                    ^^ ArgumentTypeCoercion: Argument $object_or_class of method_exists() expects 'object|string', got 'int' — coercion may fail at runtime
        /** @mir-check $_ is never */
        $_ = 1;
    }
}

function propertyExistsUnreachable(int $x): void {
    if (property_exists($x, 'foo')) {
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^ RedundantCondition: Condition of type 'bool' always evaluates the same way, so one branch is unreachable
//                      ^^ ArgumentTypeCoercion: Argument $object_or_class of property_exists() expects 'object|string', got 'int' — coercion may fail at runtime
        /** @mir-check $_ is never */
        $_ = 1;
    }
}

/** @param object|int $x */
function methodExistsStillNarrowsObject($x): void {
    if (method_exists($x, 'foo')) {
//                    ^^ PossiblyInvalidArgument: Argument $object_or_class of method_exists() expects 'object|string', possibly different type 'object|int' provided
        /** @mir-check $x is object */
        $_ = 1;
    }
}
===expect===
