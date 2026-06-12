===description===
Inherit templated mixin with self
===file===
<?php
/**
 * @template T
 */
class Mixin {
    /**
     * @var T
     */
    private $var;

    /**
     * @param T $var
     */
    public function __construct ($var) {
        $this->var = $var;
    }

    /**
     * @return T
     */
    public function type() {
        return $this->var;
    }
}

/**
 * @template T as object
 * @mixin Mixin<T>
 */
abstract class Foo {
    /** @var Mixin<T> */
    public object $obj;

    public function __call(string $name, array $args) {
        return $this->obj->$name(...$args);
    }
}

/**
 * @extends Foo<self>
 */
abstract class FooChild extends Foo{}

/**
 * @suppress MissingConstructor
 */
final class FooGrandChild extends FooChild {}

function test() : FooGrandChild {
    return (new FooGrandChild)->type();
}
===expect===
UnusedPsalmSuppress@47:0-47:0: Suppress annotation for 'MissingConstructor' is never used
