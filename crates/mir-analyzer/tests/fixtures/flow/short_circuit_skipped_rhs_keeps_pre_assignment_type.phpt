===description===
On the path that short-circuits past `$x = f()`, `$x` keeps the type it had before the condition
===file===
<?php
function find(string $n): ?string { return $n === '' ? null : $n; }

function andFalse(string $php, bool $a): string {
    if ($a && !$php = find($php)) {
        return '';
    }
    /** @mir-check $php is string */
    return $php;
}

function orTrue(string $php, bool $a): string {
    if ($a || $php = find($php)) {
        /** @mir-check $php is string */
        return $php;
    }
    return '';
}
