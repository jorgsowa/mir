===description===
No fatal error on missing class without slash
===ignore===
TODO
===file===
<?php
class Func {
    public function __construct(string $name, callable $callable) {}
}

new Func("f", ["Foo", "bar"]);
===expect===
