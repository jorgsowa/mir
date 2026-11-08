===description===
`isset()` on a key a lone sealed shape doesn't declare proves the key, so the shape was too narrow.
===file===
<?php
/** @param array{type: string} $s */
function f(array $s): string {
    if (!isset($s['url']) || !is_string($s['url'])) { throw new RuntimeException(); }
    return $s['url'];
}
