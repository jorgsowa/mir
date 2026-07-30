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
requiresObject(null);

final class C {
    /** @param object $x */
    public function requiresObject(object $x): void {}
}
(new C())->requiresObject(null);
===expect===
UnusedParam@3:24-3:33: Parameter $x is never used
NullArgument@4:15-4:19: Argument $x of requiresObject() cannot be null
UnusedParam@8:35-8:44: Parameter $x is never used
NullArgument@10:26-10:30: Argument $x of requiresObject() cannot be null
