===description===
Returns typed closure with bad return type
===file===
<?php
/**
 * @param Closure(int):int $f
 * @param Closure(int):int $g
 *
 * @return Closure(int):string
 */
function foo(Closure $f, Closure $g) : Closure {
    return function (int $x) use ($f, $g) : int {
//  ^ +2:6 InvalidReturnType: Return type 'Closure(int): int' is not compatible with declared 'Closure(int): string'
        return $f($g($x));
    };
}
