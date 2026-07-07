===description===
FN: narrowing a bare, unresolved template `T` via `instanceof`/`is_a()`/
`is_subclass_of()` replaced it outright with the checked class, discarding
the template binding instead of producing `T&Class`. Any later use relying
on the value still being "generically T" (e.g. returning it from a function
declared `@return T`) silently degraded to the concrete narrowed class. The
OR-chain form (`$x instanceof A || $x instanceof B`) has the same gap.
===config===
suppress=MissingReturnType,UnusedParam,MixedArgument
===file===
<?php
/**
 * @template T
 * @param T $x
 */
function narrowsSingle($x): void {
    if ($x instanceof Countable) {
        /** @mir-check $x is T&Countable */
        $_ = 1;
    }
}

/**
 * @template T
 * @param T $x
 */
function narrowsStrictSubclass($x): void {
    if (is_subclass_of($x, 'Countable')) {
        /** @mir-check $x is T&Countable */
        $_ = 1;
    }
}

/**
 * @template T
 * @param T $x
 */
function narrowsOrChain($x): void {
    if ($x instanceof ArrayAccess || $x instanceof Countable) {
        /** @mir-check $x is T&(ArrayAccess|Countable) */
        $_ = 1;
    }
}
===expect===
