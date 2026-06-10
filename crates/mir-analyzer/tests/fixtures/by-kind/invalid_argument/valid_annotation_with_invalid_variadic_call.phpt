===description===
Valid annotation with invalid variadic call
===file===
<?php
class ParentClass {
    public function __call(string $name, array $args) {}
}

/**
 * @method void setInts(int ...$foo) with some more text
 */
class Child extends ParentClass {}

$child = new Child();

$child->setInts([1, 2, 3]);
===expect===
