===description===
Returns typed closure with subclass return
===ignore===
TODO
===file===
<?php
class A {}
class B {}
class C {}
class A2 extends A {}

/**
 * @param Closure(B):A $f
 * @param Closure(C):B $g
 *
 * @return Closure(C):A2
 */
function foo(Closure $f, Closure $g) : Closure {
    return function (C $x) use ($f, $g) : A {
        return $f($g($x));
    };
}
===expect===
