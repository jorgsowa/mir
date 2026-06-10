===description===
Annotation with sealed
===file===
<?php
class ParentClass {
    public function __call(string $name, array $args) {}
}

/**
 * @method string getString()
 */
class Child extends ParentClass {}

$child = new Child();
$child->getString();
$child->foo();
===expect===
