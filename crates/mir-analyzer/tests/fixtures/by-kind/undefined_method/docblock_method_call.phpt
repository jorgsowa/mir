===description===
Method calls via @method docblock annotations should not trigger undefined_method
===file===
<?php

/**
 * @method string getName()
 * @method void setName(string $name)
 * @method static int create()
 */
class MyClass {
}

$obj = new MyClass();
$obj->getName();
$obj->setName("test");
MyClass::create();
===expect===
