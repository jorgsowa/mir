===description===
`is_iterable()`'s false branch must not empty out (and mark the branch
divergent) when the sole possible type is a `final` class that provably
doesn't implement `Traversable` — that type is exactly what guarantees the
condition false, so the branch stays reachable and a real bug in it is
still caught.
===file===
<?php
final class Money {
    public function __construct(public int $cents) {}
}

function describe(Money $m): void {
    if (!is_iterable($m)) {
        $m->missingMethod();
    }
}
===expect===
UndefinedMethod@8:8-8:27: Method Money::missingMethod() does not exist
