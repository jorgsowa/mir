===description===
Variant templated properties
===config===
suppress=MissingPropertyType
===file===
<?php
/**
 * @template T as string|null
 */
abstract class A {
    /** @var T */
    public $foo;
}

/**
 * @extends A<string>
 */
class AChild extends A {
    /** @var int */
    public $foo = 0;
}
===expect===
MissingConstructor@13:0-13:24: Class AChild has uninitialized properties but no constructor
