===description===
A `static $var`'s WRITE was never checked at all under any purity tag —
only the one-time declaration site fired, and only under @pure. Unlike
`global $x;`, a `static` var was never tracked anywhere, so a later
`++`/`--`/compound-op write or a plain overwrite was completely invisible
to @mutation-free/@external-mutation-free, the same class of persistent
cross-call state a static PROPERTY write already correctly flags.
===config===
suppress=MissingConstructor
===file===
<?php
class Counter {
    /** @psalm-mutation-free */
    public function tick(): int {
        static $n = 0;
        $n++;
        return $n;
    }

    /** @psalm-mutation-free */
    public function reset(): void {
        static $m = 0;
        $m = 0;
    }
}
===expect===
ImpureStaticVariable@6:8-6:10: Using static variable $n in a @pure function
UnusedVariable@12:15-12:21: Variable $m is never read
ImpureStaticVariable@13:8-13:14: Using static variable $m in a @pure function
