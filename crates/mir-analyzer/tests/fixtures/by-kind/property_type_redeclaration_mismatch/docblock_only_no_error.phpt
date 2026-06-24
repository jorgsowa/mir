===description===
Docblock-only type mismatch is not flagged — only native type hints create runtime invariant
===config===
suppress=MissingPropertyType
===file===
<?php
class A {
    /** @var int */
    public $x = 1;
}

class B extends A {
    /** @var string */
    public $x = 'hello';
}
===expect===
