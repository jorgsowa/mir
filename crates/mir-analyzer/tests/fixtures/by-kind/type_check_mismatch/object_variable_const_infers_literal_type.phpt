===description===
$obj::CONST (constant access through an object-instance variable) resolves to the constant's literal type instead of falling back to mixed.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
class Suit {
    const HEARTS = 'H';
    const MAX = 100;
}

function check(Suit $s): void {
    $h = $s::HEARTS;
    /** @mir-check $h is 'H' */
    $_ = $h;

    $m = $s::MAX;
    /** @mir-check $m is 100 */
    $_ = $m;
}
===expect===
