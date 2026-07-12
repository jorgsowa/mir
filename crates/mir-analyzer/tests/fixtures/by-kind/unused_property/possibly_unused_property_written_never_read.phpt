===description===
Possibly unused property written never read
===config===
suppress=
===file===
<?php
final class A {
    public string $foo = "hello";
}

$a = new A();
$a->foo = "bar";
===expect===
