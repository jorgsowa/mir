===description===
The false branch of `$x instanceof Closure` must exclude a `Closure(): R`-typed
atom from the value's type, the same way a TNamedObject atom is already
excluded — filter_out_instanceof_match had no TClosure arm, so the false
branch (and its reuse by is_a()'s false branch / negated instanceof) silently
kept the impossible Closure atom alive.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
class Foo {}

/**
 * @param Foo|Closure(): Foo $x
 */
function falseBranchExcludesClosure(Foo|Closure $x): void {
    if ($x instanceof Closure) {
        return;
    }
    /** @mir-check $x is Foo */
    $_ = 1;
}

/**
 * @param Foo|Closure(): Foo $x
 */
function negatedInstanceofExcludesClosure(Foo|Closure $x): void {
    if (!($x instanceof Closure)) {
        /** @mir-check $x is Foo */
        $_ = 1;
    }
}
===expect===
