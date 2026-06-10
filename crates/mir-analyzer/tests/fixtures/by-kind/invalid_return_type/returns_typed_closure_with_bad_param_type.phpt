===description===
Returns typed closure with bad param type
===ignore===
TODO
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
        return $f($g($x));
    };
}
===expect===
