===description===
readonlyPromotedPropertyAccess
===file===
<?php
class A {
    public function __construct(private readonly string $bar) {
    }
}

$a = new A("hello");
$b = $a->bar;
===expect===
InaccessibleProperty
===ignore===
TODO
