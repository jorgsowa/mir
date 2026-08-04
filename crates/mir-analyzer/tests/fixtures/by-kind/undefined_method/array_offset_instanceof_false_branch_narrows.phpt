===description===
G11, false-branch/early-return forms of `instanceof` narrowing on a
literal-keyed array-offset access. `negatedGuard` proves presence via the
negated form (`!(... instanceof Foo)` + early return); `earlyReturnOnMatch`
proves the OPPOSITE fact (returning when it DOES match `Foo` excludes `Foo`
from the fall-through, so a `Foo`-only method call afterwards must still be
flagged — proving the exclusion is real, not just "narrowing never fires").
===file===
<?php
class Foo { public function fooOnly(): void {} }
class Bar {}

/** @param array{item: Foo|Bar} $arr */
function negatedGuard(array $arr): void {
    if (!($arr['item'] instanceof Foo)) {
        return;
    }
    $arr['item']->fooOnly();
}

/** @param array{item: Foo|Bar} $arr */
function earlyReturnOnMatch(array $arr): void {
    if ($arr['item'] instanceof Foo) {
        return;
    }
    $arr['item']->fooOnly();
}
===expect===
UndefinedMethod@18:4-18:27: Method Bar::fooOnly() does not exist
