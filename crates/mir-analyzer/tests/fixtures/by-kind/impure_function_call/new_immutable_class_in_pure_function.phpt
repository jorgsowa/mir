===description===
`new X` in a `@psalm-pure` function is pure when X is `@psalm-immutable` or its
constructor is `@psalm-external-mutation-free` (also from a trait), since
constructing only initializes the new instance. A plain mutable constructor is
still impure.
===file===
<?php
/** @psalm-immutable */
final class Id {
    public function __construct(public readonly string $a) {}
}

trait UnionTrait {
    /** @psalm-external-mutation-free */
    public function __construct(public array $types) {}
}

final class Union {
    use UnionTrait;
}

final class Counter {
    public function __construct(public int $n) {}
}

/** @psalm-pure */
function make(string $a): array {
    return [new Id($a), new Union([$a])];
}

/** @psalm-pure */
function count_from(int $n): Counter {
    return new Counter($n);
//         ^^^^^^^^^^^^^^^ ImpureFunctionCall: Calling impure function Counter::__construct() in a @pure function
}
