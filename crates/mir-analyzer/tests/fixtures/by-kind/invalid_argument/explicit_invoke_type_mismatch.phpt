===description===
Explicit invoke type mismatch
===file===
<?php
class A {
    public function __invoke(string $p): void {}
}
(new A)->__invoke(1);
===expect===
InvalidArgument@5:19-5:20: Argument $p of __invoke() expects 'string', got '1'
