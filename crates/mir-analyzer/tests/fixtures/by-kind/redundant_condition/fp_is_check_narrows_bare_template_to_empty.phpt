===description===
FP: `is_string()`/`is_int()`/etc. narrowing on a variable whose declared type
is a bare, unresolved template `T` dropped it to the empty type, because none
of the `narrow_to_*` helpers treated `TTemplateParam` as "unknown, keep it"
the way they already do for `TMixed`. An empty narrowed type reads as an
unreachable branch, so every one of these checks was flagged redundant even
though `T` could resolve to anything at the call site.
===config===
suppress=MissingReturnType,UnusedParam
===file===
<?php
/**
 * @template T
 * @param T $x
 */
function checkString($x): void {
    if (is_string($x)) {
        echo "str";
    } else {
        echo "not";
    }
}

/**
 * @template T
 * @param T $x
 */
function checkInt($x): void {
    if (is_int($x)) {
        echo "int";
    }
}

/**
 * @template T
 * @param T $x
 */
function checkArray($x): void {
    if (is_array($x)) {
        echo "arr";
    }
}

/**
 * @template T
 * @param T $x
 */
function checkObject($x): void {
    if (is_object($x)) {
        echo "obj";
    }
}
===expect===
