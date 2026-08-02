===description===
Negative control for the P15 no-catch fix: WITH a catch clause present
that neither rethrows nor initializes $str, code after the try/catch/
finally statement IS reachable on the caught path without $str ever
being assigned, so the PossiblyUndefinedVariable must still fire. Confirms
the no-catch fix (scoped to `tc.catches.is_empty()`) doesn't over-correct
into a false negative when a catch clause genuinely can complete without
the variable being set.
===config===
suppress=UnusedVariable,MissingThrowsDocblock
===file===
<?php
function encode(mixed $value): void {
    try {
        $str = json_encode($value);
    } catch (\Throwable $e) {
        // does not rethrow, does not initialize $str
    } finally {
        // cleanup, does not touch $str
    }
    assert($str !== false);
}
===expect===
PossiblyUndefinedVariable@10:11-10:15: Variable $str might not be defined
