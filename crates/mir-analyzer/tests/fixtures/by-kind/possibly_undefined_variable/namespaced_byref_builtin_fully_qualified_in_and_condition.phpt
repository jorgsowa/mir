===description===
A leading-backslash by-ref builtin call defines its out param in the true
branch of `&&`.
===file===
<?php
namespace App;

function f(?string $p): ?int {
    if ($p !== null && \preg_match('/(\d+)/', $p, $m) === 1) {
        return (int) $m[1];
    }
    return null;
}
