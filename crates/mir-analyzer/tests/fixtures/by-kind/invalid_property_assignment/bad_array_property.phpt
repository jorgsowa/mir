===description===
Bad array property
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
MissingConstructor@6:0-6:9: Class C has uninitialized properties but no constructor
