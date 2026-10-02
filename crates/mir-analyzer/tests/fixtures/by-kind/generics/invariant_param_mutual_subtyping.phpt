===description===
Invariant template args compare by mutual subtyping, so equivalent spellings
(`int<0, max>`, `non-negative-int`, literal-absorbed unions) are accepted while
a genuinely different arg is still rejected.
===config===
suppress=UnusedVariable,UnusedParam,MissingParamType,MixedAssignment,MissingConstructor,UnnecessaryVarAnnotation
===file===
<?php
/** @template T */
final class Cached {
    /** @param \Closure(): T $f */
    public function __construct(private \Closure $f) {}
}

final class Holder {
    /** @var Cached<int<0, max>> */
    private Cached $v;
    public function ternary(bool $cond, int $x): void {
        $this->v = new Cached(fn() => $cond ? max(0, $x) : 5);
    }

    public function named(): void {
        /** @var Cached<non-negative-int> $c */
        $c = new Cached(fn() => 1);
        $this->v = $c;
    }

    public function rejected(): void {
        /** @var Cached<int> $c */
        $c = new Cached(fn() => 1);
        $this->v = $c;
//      ^^^^^^^^^^^^^ InvalidPropertyAssignment: Property $v expects 'Cached<int<0, max>>', cannot assign 'Cached<int>'
    }
}
===expect===
