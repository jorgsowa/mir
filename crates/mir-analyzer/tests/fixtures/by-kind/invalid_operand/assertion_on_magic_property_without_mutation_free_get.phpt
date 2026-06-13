===description===
Assertion on magic property without mutation free get
===config===
suppress=UnusedParam
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

/** @assert-if-true  string $arg->b */
function assertString(A $arg): bool {return $arg->b !== null;}

if (assertString($a)) {
    requiresString($a->b);
}

function requiresString(string $_str): void {}

===expect===
PossiblyNullArgument@16:20-16:25: Argument $_str of requiresString() might be null
