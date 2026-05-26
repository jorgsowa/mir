===description===
Annotation invalid arg
===file===
<?php
class ParentClass {
    public function __call(string $name, array $args) {}
}

/**
 * @method setString(int $integer)
 */
class Child extends ParentClass {}

$child = new Child();

$child->setString("five");
===expect===
InvalidArgument@13:19: Argument $integer of setString() expects 'int', got '"five"'
