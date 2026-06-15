===description===
Missing trait property type
===file===
<?php
trait T {
    public $foo = 5;
}

class A {
    use T;
}
===expect===
MissingPropertyType@3:4-3:19: Property T::$foo has no type annotation
