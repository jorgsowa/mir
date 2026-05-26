===description===
Prevent ternary passed by reference
===file===
<?php
/**
 * @param string $p
 */
function b(&$p): string {
    return $p;
}

function main(bool $a, string $b, string $c): void {
    b($a ? $b : $c);
}
===expect===
InvalidPassByReference
===ignore===
TODO
