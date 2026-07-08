===description===
@readonly docblock (advisory, not runtime-enforced) dropped in child — no error
===config===
suppress=MissingPropertyType
===file===
<?php
class A {
    /** @readonly */
    public $x = 0;
}

class B extends A {
    public $x = 1;
}
===expect===
