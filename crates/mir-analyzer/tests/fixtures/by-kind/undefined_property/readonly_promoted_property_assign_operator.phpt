===description===
Readonly promoted property assign operator
===ignore===
TODO
===file===
<?php
class A {
    public function __construct(public readonly string $bar) {
    }
}

$a = new A("hello");
$a->bar = "goodbye";
===expect===
