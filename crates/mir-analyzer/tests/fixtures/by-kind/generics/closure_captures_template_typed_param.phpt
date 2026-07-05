===description===
G1: a closure/arrow function must inherit the enclosing function's template
param names. A captured @template-typed variable assigned to a concretely
typed property inside a closure must not be flagged as InvalidPropertyAssignment
— the same assignment written directly in the enclosing function is correctly
allowed since the template placeholder isn't provably incompatible.
===config===
suppress=UnusedVariable,MissingReturnType,MissingPropertyType,MissingClosureReturnType
===file===
<?php
class Box {
    public int $n = 0;
}

/**
 * @template T
 * @param T $x
 */
function use_in_closure($x): void {
    $box = new Box();
    $fn = function () use ($x, $box) {
        $box->n = $x;
    };
    $fn();
}

/**
 * @template T
 * @param T $x
 */
function use_in_arrow($x): void {
    $box = new Box();
    $set = fn() => $box->n = $x;
    $set();
}
===expect===
