===description===
A by-ref builtin called unqualified from a namespaced file defines its out
param in the true branch of `&&`, same as in the global namespace.
===file===
<?php
namespace App;

function f(?string $p): ?int {
    if ($p !== null && preg_match('/(\d+)/', $p, $m) === 1) {
        /** @mir-check $m is list<string> */
        return (int) $m[1];
    }
    return null;
}
