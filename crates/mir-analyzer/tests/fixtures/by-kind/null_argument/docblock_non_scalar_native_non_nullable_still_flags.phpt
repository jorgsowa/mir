===description===
Negative control for the docblock-non-scalar-nullability fix: when the
native hint is genuinely NOT nullable, passing null still flags — the fix
only preserves nullability the native hint actually declares, it doesn't
add leniency the hint doesn't have. Covers both a free function and a
method, since each has its own param-merging code path.
===file===
<?php
/** @param object $x */
function requiresObject(object $x): void {}
//                      ^^^^^^^^^ UnusedParam: Parameter $x is never used
requiresObject(null);
//             ^^^^ NullArgument: Argument $x of requiresObject() cannot be null

final class C {
    /** @param object $x */
    public function requiresObject(object $x): void {}
//                                 ^^^^^^^^^ UnusedParam: Parameter $x is never used
}
(new C())->requiresObject(null);
//                        ^^^^ NullArgument: Argument $x of requiresObject() cannot be null
