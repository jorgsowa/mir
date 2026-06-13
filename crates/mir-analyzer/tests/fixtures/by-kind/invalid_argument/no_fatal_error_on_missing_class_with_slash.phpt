===description===
No fatal error on missing class with slash
===config===
suppress=UnusedParam
===file===
<?php
class Func {
    public function __construct(string $name, callable $callable) {}
}

new Func("f", ["Foo", "bar"]);
===expect===
