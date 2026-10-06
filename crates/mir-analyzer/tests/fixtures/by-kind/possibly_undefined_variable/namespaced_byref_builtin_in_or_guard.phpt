===description===
A guard clause `$p === null || preg_match(...) !== 1` defines the out param
after the early return, in a namespaced file.
===file===
<?php
namespace App;

function f(?string $p): ?int {
    if ($p === null || preg_match('/(\d+)/', $p, $m) !== 1) {
        return null;
    }
    /** @mir-check $m is list<string> */
    return (int) $m[1];
}
