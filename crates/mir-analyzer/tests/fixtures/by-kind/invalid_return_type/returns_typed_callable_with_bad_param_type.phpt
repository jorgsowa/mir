===description===
Returns typed callable with bad param type
===ignore===
TODO
===file===
<?php
/**
 * @param Closure(int):int $f
 * @param Closure(int):int $g
 *
 * @return callable(string):int
 */
function foo(Closure $f, Closure $g) : callable {
    return function (int $x) use ($f, $g) : int {
        return $f($g($x));
    };
}
===expect===
