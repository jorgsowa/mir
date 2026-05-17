===description===
@method docblocks on interfaces should be available on implementing classes
===file===
<?php

/**
 * @method string getName()
 */
interface MyInterface {
}

class MyClass implements MyInterface {
}

$obj = new MyClass();
$obj->getName();
===expect===
