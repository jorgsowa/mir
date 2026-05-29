===description===
Union annotation invalid arg
===file===
<?php
class ParentClass {
    public function __call(string $name, array $args) {}
}

/**
 * @method setBool(string $foo, string|bool $bar)  :   bool dsa sada
 */
class Child extends ParentClass {}

$child = new Child();

$b = $child->setBool("hello", 5);
===expect===
InvalidArgument@13:31-13:32: Argument $bar of setBool() expects 'string|bool', got '5'
