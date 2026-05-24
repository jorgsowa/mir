===description===
callableMissingOptionalVariableInstanceArray
===file===
<?php
/**
 * @param callable(string=):bool $arg
 * @return void
 */
function foo($arg) {}

class A {
    public function bar(): bool {
        return true;
    }
}

$a_instance = new A();
$y = [$a_instance, "bar"];
foo($y);
===expect===
PossiblyInvalidArgument
===ignore===
TODO
