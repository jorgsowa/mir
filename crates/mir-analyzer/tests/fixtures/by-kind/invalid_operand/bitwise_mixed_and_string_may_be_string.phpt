===description===
A bitwise operator on a `mixed` and a `string` operand may give a string.
===file===
<?php
/** @param mixed $m */
function f($m, string $s): void {
    $r = $m & $s;
    /** @mir-check $r is int|string */
    echo $r;
}
