===description===
returnsTypedClosureWithBadCall
===file===
<?php
class A {}
class B {}
class C {}
class D {}

/**
 * @param Closure(B):A $f
 * @param Closure(C):B $g
 *
 * @return Closure(C):A
 */
function foo(Closure $f, Closure $g) : Closure {
    return function (int $x) use ($f, $g) : int {
        return $f($g($x));
    };
}
===expect===
InvalidReturnType@15:8: Return type 'A' is not compatible with declared 'int'
