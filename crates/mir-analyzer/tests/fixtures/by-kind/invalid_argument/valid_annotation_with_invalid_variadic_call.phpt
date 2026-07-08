===description===
Valid @method annotation, but the call passes a plain array literal (no
spread) to a variadic int param — binds as $foo[0] = array, not three ints
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
InvalidArgument@13:16-13:25: Argument $foo of setInts() expects 'int', got 'array{0: 1, 1: 2, 2: 3}'
