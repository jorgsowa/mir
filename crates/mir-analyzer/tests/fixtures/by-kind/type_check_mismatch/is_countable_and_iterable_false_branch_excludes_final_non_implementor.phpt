===description===
is_countable()/is_iterable()'s false branch now also excludes a named-object
atom when it's `final` (no subclass could add `implements
Countable`/`Traversable` later) AND its own hierarchy provably doesn't
already implement it — beyond the pre-existing array-atom exclusion. A
non-final class (unknown subclasses might implement the interface) and a
final class that already implements the interface both stay, matching the
same final-class soundness gate narrow_var_to_specific_class's false branch
uses for exact-class exclusion.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
final class PlainFinal {}

final class CountableFinal implements Countable {
    public function count(): int { return 0; }
}

class NonFinalPlain {}

/** @param PlainFinal|NonFinalPlain|array<int,int> $x */
function test_is_countable_false_excludes_final_non_countable(mixed $x): void {
    if (!is_countable($x)) {
        /** @mir-check $x is NonFinalPlain */
        $_ = $x;
    }
}

/** @param PlainFinal|NonFinalPlain|array<int,int> $x */
function test_is_iterable_false_excludes_final_non_traversable(mixed $x): void {
    if (!is_iterable($x)) {
        /** @mir-check $x is NonFinalPlain */
        $_ = $x;
    }
}

/** @param CountableFinal|NonFinalPlain|array<int,int> $x */
function test_is_countable_false_keeps_final_implementor(mixed $x): void {
    if (!is_countable($x)) {
        /** @mir-check $x is CountableFinal|NonFinalPlain */
        $_ = $x;
    }
}
===expect===
