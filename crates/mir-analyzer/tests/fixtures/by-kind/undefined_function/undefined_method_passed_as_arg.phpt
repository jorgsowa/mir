===description===
Undefined method passed as arg
===file===
<?php
class A {
    public function __call(string $method, array $args) {}
}

$q = new A;
$q->foo(bar());
===expect===
UndefinedFunction@7:8-7:13: Function bar() is not defined
