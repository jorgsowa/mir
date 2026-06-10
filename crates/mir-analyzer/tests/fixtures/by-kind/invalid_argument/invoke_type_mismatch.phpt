===description===
Invoke type mismatch
===ignore===
TODO
===file===
<?php
class A {
    public function __invoke(string $p): void {}
}

$q = new A;
$q(1);
===expect===
