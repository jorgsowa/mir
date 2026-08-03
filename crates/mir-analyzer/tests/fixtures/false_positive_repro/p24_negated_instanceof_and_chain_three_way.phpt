===description===
P24 sibling: the same ANDed negated-instanceof guard-clause narrowing must
recurse through a 3-way chain (`(!A && !B) && !C`), not just a single `&&`.
===file===
<?php

final class Good1 { public function file(): string { return 'x'; } }
final class Good2 { public function file(): string { return 'x'; } }
final class Good3 { public function file(): string { return 'x'; } }
final class Bad1 {}

function reasonLocation(Good1|Good2|Good3|Bad1 $reason): string
{
    if (!$reason instanceof Good1 && !$reason instanceof Good2 && !$reason instanceof Good3) {
        return '';
    }
    return $reason->file();
}
===expect===
