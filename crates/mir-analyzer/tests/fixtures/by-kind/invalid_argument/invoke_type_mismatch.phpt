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
