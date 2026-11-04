===description===
reset(), end(), next() and prev() move the internal pointer without changing the array's type
===file===
<?php
/** @param list<string> $a */
function f(array $a): void {
    end($a);
    reset($a);
    next($a);
    prev($a);
    /** @mir-check $a is list<string> */
    $a[] = end($a) . 'x';
    /** @mir-check $a is list<string> */
    echo 1;
}
