===description===
P3: A typed first-class method callable satisfies a callable(int):string typed param
without false positives.
===config===
suppress=UnusedVariable
===file===
<?php

class Converter {
    public function toString(int $x): string { return (string) $x; }
}

/**
 * @param callable(int): string $fn
 */
function apply(callable $fn, int $x): string {
    return $fn($x);
}

$c = new Converter();
apply($c->toString(...), 42);
===expect===
