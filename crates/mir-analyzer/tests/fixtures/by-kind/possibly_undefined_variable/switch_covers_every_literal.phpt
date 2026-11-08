===description===
A switch with a `case` for every value of a literal-union subject always enters a case.
===file===
<?php
/** @param 'y'|'m'|1 $k */
function f($k): string {
    switch ($k) {
        case 'y': $s = 'year'; break;
        case 'm': $s = 'month'; break;
        case 1: $s = 'one'; break;
    }
    return $s;
}
/** @param 'y'|'m' $k */
function g($k): string {
    switch ($k) {
        case 'y': $s = 'year'; break;
    }
    return $s;
//         ^^ PossiblyUndefinedVariable: Variable $s might not be defined
}
