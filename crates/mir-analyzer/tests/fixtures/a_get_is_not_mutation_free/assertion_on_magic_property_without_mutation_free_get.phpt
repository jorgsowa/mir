===description===
assertionOnMagicPropertyWithoutMutationFreeGet
===file===
<?php
/**
 * @property ?string $b
 */
class A {
    public function __get(string $key) {return "";}
    public function __set(string $key, string $value): void {}
}

$a = new A;

/** @psalm-assert-if-true  string $arg->b */
function assertString(A $arg): bool {return $arg->b !== null;}

if (assertString($a)) {
    requiresString($a->b);
}

function requiresString(string $_str): void {}

===expect===
A::__get is not mutation-free
===ignore===
TODO
