===description===
noFatalErrorOnMissingClassWithSlash
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
