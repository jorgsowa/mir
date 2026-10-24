===description===
`instanceof` resolves fully qualified and relative class names, for defined and undefined classes.
===file===
<?php
namespace App;

class A { public function onlyA(): int { return 1; } }

function f(object $o): int {
    if ($o instanceof \App\A) {
        return $o->onlyA();
    }
    if ($o instanceof namespace\A) {
        return $o->onlyA();
    }
    return 0;
}

function g(object $o): bool {
    return $o instanceof \App\Missing || $o instanceof namespace\Missing;
//                       ^^^^^^^^^^^^ UndefinedClass: Class App\Missing does not exist
//                                                     ^^^^^^^^^^^^^^^^^ UndefinedClass: Class App\Missing does not exist
}
