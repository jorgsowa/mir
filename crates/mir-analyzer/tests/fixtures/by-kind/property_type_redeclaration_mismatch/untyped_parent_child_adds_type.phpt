===description===
Child adds a native type hint where parent had none — PHP allows this, no error
===config===
suppress=MissingPropertyType
===file===
<?php
class A {
    public $x = 1;
}

class B extends A {
    public int $x = 2;
}
===expect===
