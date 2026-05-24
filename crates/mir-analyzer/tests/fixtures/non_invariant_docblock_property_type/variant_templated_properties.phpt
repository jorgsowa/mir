===description===
variantTemplatedProperties
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
NonInvariantDocblockPropertyType
===ignore===
TODO
