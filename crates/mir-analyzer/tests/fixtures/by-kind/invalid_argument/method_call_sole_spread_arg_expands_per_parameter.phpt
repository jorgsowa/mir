===description===
A sole spread argument over a literal, sequentially-keyed shape must be
expanded into one binding per element on an instance METHOD call too, not
just function/static-call/constructor calls — both for per-parameter
argument checking and for template inference from the call's own args.
===config===
suppress=UnusedParam,UnusedVariable
===file===
<?php
class Calc {
    public function needsTwoInts(int $a, int $b): void {}
}

/** @param array{0: int, 1: string} $pair */
function checksEachArg(Calc $c, array $pair): void {
    $c->needsTwoInts(...$pair);
}

class Pair {
    /**
     * @template A
     * @template B
     * @param A $first
     * @param B $second
     * @return array{0: A, 1: B}
     */
    public function make($first, $second): array {
        return [$first, $second];
    }
}

function infersTemplatesFromSpread(Pair $p): void {
    $result = $p->make(...['x', 42]);
    /** @mir-check $result is array{0: "x", 1: 42} */
    $_ = 1;
}
===expect===
InvalidArgument@8:22-8:29: Argument $b of needsTwoInts() expects 'int', got 'string'
