===description===
@method docblocks should work with inheritance
===file===
<?php

/**
 * @method string getName()
 */
class ParentClass {
}

class Child extends ParentClass {
}

$obj = new Child();
$obj->getName();
===expect===
