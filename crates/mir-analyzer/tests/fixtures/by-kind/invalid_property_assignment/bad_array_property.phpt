===description===
Bad array property
===config===
suppress=MissingPropertyType
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
