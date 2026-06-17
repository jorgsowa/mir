===description===
Invoke type mismatch
===config===
suppress=UnusedParam
===file===
<?php
class A {
    public function __invoke(string $p): void {}
}

$q = new A;
$q(1);
===expect===
ArgumentTypeCoercion@7:3-7:4: Argument $p of A::__invoke() expects 'string', got '1' — coercion may fail at runtime
