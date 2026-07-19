===description===
is_countable()/is_iterable()'s false branch excludes a named-object atom
that already provably extends/implements Countable/Traversable (guaranteed
true, so it can never survive into the false branch). A `final` class that
provably does NOT implement the interface is guaranteed false and so is
kept, and a non-final class (unknown subclasses might implement it) is also
kept.
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
function test_is_countable_false_keeps_final_non_countable(mixed $x): void {
    if (!is_countable($x)) {
        /** @mir-check $x is PlainFinal|NonFinalPlain */
        $_ = $x;
    }
}

/** @param PlainFinal|NonFinalPlain|array<int,int> $x */
function test_is_iterable_false_keeps_final_non_traversable(mixed $x): void {
    if (!is_iterable($x)) {
        /** @mir-check $x is PlainFinal|NonFinalPlain */
        $_ = $x;
    }
}

/** @param CountableFinal|NonFinalPlain|array<int,int> $x */
function test_is_countable_false_excludes_final_implementor(mixed $x): void {
    if (!is_countable($x)) {
        /** @mir-check $x is NonFinalPlain */
        $_ = $x;
    }
}
===expect===
