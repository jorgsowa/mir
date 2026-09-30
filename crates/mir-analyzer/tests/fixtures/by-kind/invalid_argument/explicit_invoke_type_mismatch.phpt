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
//                ^ ArgumentTypeCoercion: Argument $p of __invoke() expects 'string', got '1' — coercion may fail at runtime
===expect===
