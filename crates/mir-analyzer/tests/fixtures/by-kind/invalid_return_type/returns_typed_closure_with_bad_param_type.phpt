===description===
Returns typed closure with bad param type
===file===
<?php
/**
 * @param Closure(int):int $f
 * @param Closure(int):int $g
 *
 * @return Closure(string):int
 */
function foo(Closure $f, Closure $g) : Closure {
    return function (int $x) use ($f, $g) : int {
//  ^ +2:6 InvalidReturnType: Return type 'Closure(int): int' is not compatible with declared 'Closure(string): int'
        return $f($g($x));
    };
}
