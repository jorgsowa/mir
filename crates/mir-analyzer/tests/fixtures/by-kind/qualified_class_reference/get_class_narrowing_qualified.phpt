===description===
`get_class($x) === Name::class` narrows through fully qualified and relative names.
===file===
<?php
namespace App;

class A { public function onlyA(): int { return 1; } }

function f(object $o): int {
    if (get_class($o) === \App\A::class) {
        return $o->onlyA();
    }
    if (get_class($o) === namespace\A::class) {
        return $o->onlyA();
    }
    return 0;
}
