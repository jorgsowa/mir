===description===
Invoke type mismatch
===file===
<?php
class A {
    public function __invoke(string $p): void {}
}

$q = new A;
$q(1);
===expect===
InvalidArgument@7:4-7:5: Argument $p of A::__invoke() expects 'string', got '1'
