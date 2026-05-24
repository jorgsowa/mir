===description===
inheritTemplatedMixinWithSelf
===file===
<?php
/**
 * @template T
 */
class Mixin {
    /**
     * @psalm-var T
     */
    private $var;

    /**
     * @psalm-param T $var
     */
    public function __construct ($var) {
        $this->var = $var;
    }

    /**
     * @psalm-return T
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
 * @psalm-suppress MissingConstructor
 */
final class FooGrandChild extends FooChild {}

function test() : FooGrandChild {
    return (new FooGrandChild)->type();
}
===expect===
LessSpecificReturnStatement
===ignore===
TODO
