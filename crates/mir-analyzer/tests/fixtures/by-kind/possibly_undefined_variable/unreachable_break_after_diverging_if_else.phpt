===description===
An unreachable `break` after an if/else that returns on both branches doesn't add a path out of the switch.
===file===
<?php
function f(int $k): string {
    switch ($k) {
        case 1:
            if ($k > 0) { return 'a'; } else { return 'b'; }
            break;
//          ^^^^^^ UnreachableCode: Unreachable code detected
        default:
            $x = 'c';
    }
    return $x;
}
