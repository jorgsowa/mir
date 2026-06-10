===description===
Bad array property
===ignore===
TODO
===file===
<?php
class A {}

class B {}

class C {
    /** @var array<B> */
    public $bb;
}

$c = new C;
$c->bb = [new A, new B];
===expect===
