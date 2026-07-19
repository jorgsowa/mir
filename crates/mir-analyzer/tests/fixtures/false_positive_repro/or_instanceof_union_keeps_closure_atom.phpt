===description===
`$x instanceof Closure || $x instanceof Foo` (OR-chain across 2+ classes)
must preserve a `Closure(): R`-typed union member — narrow_or_instanceof_union
had no TClosure arm, so a real Closure-typed value was silently dropped from
the merged type, producing a false-positive RedundantCondition on an inner
re-check that's actually reachable. `Bar` keeps the outer OR non-exhaustive
so only the inner re-check's behavior is under test.
===config===
suppress=UnusedVariable,UnusedParam,MissingConstructor
===file===
<?php
class Foo {}
class Bar {}

/**
 * @param Foo|Bar|Closure(): Foo $x
 */
function orChainKeepsClosure(Foo|Bar|Closure $x): ?Foo {
    if ($x instanceof Closure || $x instanceof Foo) {
        if ($x instanceof Closure) {
            return $x();
        }
        return $x;
    }
    return null;
}

class Holder {
    /** @var Foo|Bar|Closure(): Foo */
    public Foo|Bar|Closure $x;
}

function orChainKeepsClosureProp(Holder $h): void {
    if ($h->x instanceof Closure || $h->x instanceof Foo) {
        if ($h->x instanceof Closure) {
            /** @mir-check $h->x is Closure(): mixed */
            $_ = 1;
        } else {
            /** @mir-check $h->x is Foo */
            $_ = 1;
        }
    }
}
===expect===
