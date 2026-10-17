===description===
A `static $var` in a @pure METHOD is reported once, at its declaration.
@pure on a method also sets the mutation-free flag, so the later write
was flagged a second time by the @mutation-free write check (seen on
brick/math's memoized `BigInteger::zero()`), while the same code in a
@pure function was reported only once.
===config===
<mir>
  <issueHandlers>
    <MissingConstructor errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
final class Num {
    /** @pure */
    public static function zero(): Num {
        /** @var Num|null $zero */
        static $zero;
//             ^^^^^ ImpureStaticVariable: Using static variable $zero in a @pure function
        if ($zero === null) {
            $zero = new Num();
        }
        return $zero;
    }

    /** @pure */
    public function tick(): int {
        static $n = 0;
//             ^^^^^^ ImpureStaticVariable: Using static variable $n in a @pure function
        $n++;
        return $n;
    }
}
