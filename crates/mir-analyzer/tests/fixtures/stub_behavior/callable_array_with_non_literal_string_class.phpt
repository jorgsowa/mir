===description===
`[$string, 'method']` is accepted as a callable; only a literal first element needs to name a class
===file===
<?php
/** @param list<string> $v */
function f(array $v, string $cls): array {
    return array_map([$cls, 'fromString'], $v);
}
