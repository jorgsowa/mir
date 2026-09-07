===description===
P24: a chain of ANDed negated `instanceof` checks before an early return must
narrow the fall-through type by De Morgan — `!$x instanceof A && !$x instanceof
B` being false means `$x instanceof A || $x instanceof B`, excluding every
other union member even though neither is named in the chain.
===file===
<?php

final class Good1 { public function file(): string { return 'x'; } }
final class Good2 { public function file(): string { return 'x'; } }
final class Bad1 {}
final class Bad2 {}

function reasonLocation(Good1|Good2|Bad1|Bad2 $reason): string
{
    if (!$reason instanceof Good1 && !$reason instanceof Good2) {
        return '';
    }
    return $reason->file();
}
===expect===
