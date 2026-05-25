===description===
Callable missing optional this array
===file===
<?php
/**
 * @param callable(string=):bool $arg
 * @return void
 */
function foo($arg) {}

class A {
    public function __construct() {
        foo([$this, "bar"]);
    }

    public function bar(): bool {
        return true;
    }
}
===expect===
PossiblyInvalidArgument
===ignore===
TODO
