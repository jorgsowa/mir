===description===
Explicit invoke type mismatch
===file===
<?php
class A {
    public function __invoke(string $p): void {}
}
(new A)->__invoke(1);
===expect===
InvalidScalarArgument
===ignore===
TODO
