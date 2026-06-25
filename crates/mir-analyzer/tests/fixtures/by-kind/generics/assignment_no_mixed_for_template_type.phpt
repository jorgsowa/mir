===description===
G1: assigning a template-typed expression to a variable must not emit MixedAssignment —
a template param V is an intentionally parameterised placeholder, not truly mixed.
===config===
suppress=UnusedVariable,MissingReturnType,MissingPropertyType
===file===
<?php
/**
 * @template T
 * @param T $value
 * @return T
 */
function identity($value) { return $value; }

/**
 * @template T
 * @param T $a
 * @param T $b
 */
function pair_assign($a, $b): void {
    $copy = $a;   // type T — must not fire MixedAssignment
    $copy2 = $b;  // type T — must not fire MixedAssignment
    $_ = $copy;
    $_ = $copy2;
}

/**
 * @template T of object
 * @param T $obj
 * @return T
 */
function wrap_and_return($obj) {
    $local = $obj; // type T of object — must not fire MixedAssignment
    return $local;
}
===expect===
