===description===
Closure->call() returns closure return type (not nullable)
===file===
<?php
class C {
    public function name(): string { return "test"; }
}

$closure = function(): array { return ["a", "b"]; };
/** @mir-check $closure is Closure(): array */

$result = $closure->call(new C());
// call() returns the closure's return type directly (not nullable since it executes immediately)
/** @mir-check $result is array */
===expect===
