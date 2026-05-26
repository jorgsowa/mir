===description===
@method docblocks on traits should be available on classes using the trait
===file===
<?php

/**
 * @method string getName()
 * @method static int create()
 */
trait MyTrait {
}

class MyClass {
    use MyTrait;
}

$obj = new MyClass();
$obj->getName();
MyClass::create();
===expect===
