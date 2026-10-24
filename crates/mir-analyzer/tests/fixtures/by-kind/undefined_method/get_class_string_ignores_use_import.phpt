===description===
A class name in a string is fully qualified at runtime, so `get_class($o) === 'Thing'`
narrows to the global `Thing` even with `use Lib\Thing` in scope; same for `'\Thing'`
and `get_debug_type()`.
===file:Global.php===
<?php
class Thing { public function globalOnly(): void {} }
===file:App.php===
<?php
namespace Lib {
    class Thing { public function libOnly(): void {} }
}
namespace App {
    use Lib\Thing;

    function f(object $o): void {
        if (get_class($o) === 'Thing') {
            $o->globalOnly();
            $o->libOnly();
//          ^^^^^^^^^^^^^ UndefinedMethod: Method Thing::libOnly() does not exist
        }
        if ('\Thing' === get_class($o)) {
            $o->globalOnly();
        }
        if (get_debug_type($o) === 'Thing') {
            $o->globalOnly();
        }
    }
}
