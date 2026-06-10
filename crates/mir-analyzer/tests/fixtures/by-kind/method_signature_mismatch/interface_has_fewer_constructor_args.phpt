===description===
Interface has fewer constructor args
===ignore===
TODO
===file===
<?php
interface Foo {
    public function __construct();
}

class Bar implements Foo {
    public function __construct(bool $foo) {}
}
===expect===
