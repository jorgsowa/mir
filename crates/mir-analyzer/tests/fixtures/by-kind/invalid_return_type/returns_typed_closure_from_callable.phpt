===description===
Returns typed closure from callable
===file===
<?php
class A {}
class B {}
class C {}

/**
 * @param Closure(B):A $f
 * @param Closure(C):B $g
 *
 * @return callable(C):A
 */
function foo(Closure $f, Closure $g) : callable {
    return function (C $x) use ($f, $g) : A {
        return $f($g($x));
    };
}

/**
 * @param Closure(B):A $f
 * @param Closure(C):B $g
 *
 * @return Closure(C):A
 */
function bar(Closure $f, Closure $g) : Closure {
    return foo($f, $g);
}
===expect===
