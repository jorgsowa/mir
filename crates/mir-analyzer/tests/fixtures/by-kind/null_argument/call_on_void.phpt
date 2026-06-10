===description===
Call on void
===ignore===
TODO
===file===
<?php
class A {
    public function foo(): void {}
}

$p = new A();
$p->foo()->bar();
===expect===
