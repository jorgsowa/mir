===description===
Returns typed closure with subclass param
===ignore===
TODO
===file===
<?php
class A {}
class B {}
class C {}
class C2 extends C {}

/**
 * @param Closure(B):A $f
 * @param Closure(C):B $g
 *
 * @return Closure(C):A
 */
function foo(Closure $f, Closure $g) : Closure {
    return function (C2 $x) use ($f, $g) : A {
        return $f($g($x));
    };
}
===expect===
