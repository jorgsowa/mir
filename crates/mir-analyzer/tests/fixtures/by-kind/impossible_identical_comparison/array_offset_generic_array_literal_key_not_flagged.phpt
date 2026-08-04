===description===
L3: a literal-keyed read off a generic `array<K,V>` may miss at runtime
(PHP's undefined-offset warning, then `null`) even though the array's
declared value type doesn't include `null` — a defensive `!== null`/
`=== null` check against it must not be flagged as an impossible
comparison, since the key's presence was never actually proven.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
/** @param array<int, string> $map */
function notIdenticalDirection(array $map): void {
    $v = $map[5];
    if ($v !== null) {}
}

/** @param array<int, string> $map */
function identicalDirection(array $map): void {
    $v = $map[5];
    if ($v === null) {}
}
===expect===
