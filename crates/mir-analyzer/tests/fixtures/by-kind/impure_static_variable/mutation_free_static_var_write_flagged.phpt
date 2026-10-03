===description===
A `static $var`'s WRITE was never checked at all under any purity tag —
only the one-time declaration site fired, and only under @pure. Unlike
`global $x;`, a `static` var was never tracked anywhere, so a later
`++`/`--`/compound-op write or a plain overwrite was completely invisible
to @mutation-free/@external-mutation-free, the same class of persistent
cross-call state a static PROPERTY write already correctly flags.
===config===
<mir>
  <issueHandlers>
    <MissingConstructor errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Counter {
    /** @psalm-mutation-free */
    public function tick(): int {
        static $n = 0;
        $n++;
//      ^^ ImpureStaticVariable: Using static variable $n in a @pure function
        return $n;
    }

    /** @psalm-mutation-free */
    public function reset(): void {
        static $m = 0;
//             ^^^^^^ UnusedVariable: Variable $m is never read
        $m = 0;
//      ^^^^^^ ImpureStaticVariable: Using static variable $m in a @pure function
    }
}
===expect===
