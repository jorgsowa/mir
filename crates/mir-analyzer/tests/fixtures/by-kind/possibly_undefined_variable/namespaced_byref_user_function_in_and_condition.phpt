===description===
A namespaced user function with a by-ref param defines the out variable in
the true branch of `&&`.
===file===
<?php
namespace App;

function fill(string $in, ?string &$out): bool {
    $out = $in;
    return true;
}

function f(?string $p): ?string {
    if ($p !== null && fill($p, $out)) {
        /** @mir-check $out is string|null */
        return $out;
    }
    return null;
}
