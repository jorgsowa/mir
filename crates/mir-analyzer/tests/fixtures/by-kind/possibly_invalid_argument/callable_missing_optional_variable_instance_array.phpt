===description===
Callable missing optional variable instance array
===config===
suppress=UnusedParam
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
