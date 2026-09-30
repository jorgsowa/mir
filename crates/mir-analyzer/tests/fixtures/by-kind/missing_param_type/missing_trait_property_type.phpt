===description===
Missing trait property type
===file===
<?php
trait T {
    public $foo = 5;
//  ^^^^^^^^^^^^^^^ MissingPropertyType: Property T::$foo has no type annotation
}

class A {
    use T;
}
===expect===
