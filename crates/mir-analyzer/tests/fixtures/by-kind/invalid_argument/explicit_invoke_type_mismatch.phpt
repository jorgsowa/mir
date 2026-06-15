===description===
Explicit invoke type mismatch
===config===
suppress=UnusedParam
===file===
<?php
class A {
    public function __invoke(string $p): void {}
}
(new A)->__invoke(1);
===expect===
InvalidArgument@5:18-5:19: Argument $p of __invoke() expects 'string', got '1'
