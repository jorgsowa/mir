===description===
Union annotation invalid arg
===config===
suppress=MixedAssignment,UnusedVariable
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
ArgumentTypeCoercion@13:30-13:31: Argument $bar of setBool() expects 'string|bool', got '5' — coercion may fail at runtime
