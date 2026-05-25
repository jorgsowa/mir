===description===
No fatal error on missing class without slash
===file===
<?php
class Func {
    public function __construct(string $name, callable $callable) {}
}

new Func("f", ["Foo", "bar"]);
===expect===
InvalidArgument
===ignore===
TODO
