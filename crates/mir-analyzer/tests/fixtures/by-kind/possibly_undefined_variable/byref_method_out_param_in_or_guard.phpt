===description===
A guard clause `$p === null || !$p->parse(...)` defines the method's out
param after the early return.
===file===
<?php
final class Parser {
    public function parse(string $in, ?int &$out): bool {
        $out = (int) $in;
        return true;
    }
}

function f(string $in, ?Parser $p): int {
    if ($p === null || !$p->parse($in, $out)) {
        return 0;
    }
    return $out ?? 0;
}
