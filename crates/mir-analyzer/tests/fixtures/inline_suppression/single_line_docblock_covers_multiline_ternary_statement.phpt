===description===
A single-line `@psalm-suppress` docblock covers every line of the statement that follows, including ternary branches on later lines, with no `UnusedSuppress`.
===file===
<?php
function takeInt(int $x): void { echo $x; }

function expressionStatement(bool $c): void {
    /** @psalm-suppress InvalidArgument */
    $c
        ? takeInt('a')
        : takeInt('b');
}

function assignment(bool $c): void {
    /** @psalm-suppress InvalidArgument */
    $r = $c
        ? takeInt('a')
        : takeInt('b');
    echo $r === null;
}

function methodChain(Chain $chain): void {
    /** @psalm-suppress InvalidArgument */
    $chain
        ->first()
        ->second('x');
}

function nestedBranches(Chain $chain): void {
    /** @psalm-suppress InvalidArgument trailing prose */
    $chain->first()
        ? $chain->second(
            'x',
        )
        : $chain->second(
            'y',
        );
}

class Chain {
    public function first(): static { return $this; }
    public function second(int $n): static { echo $n; return $this; }
}
