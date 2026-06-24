===description===
Without @inheritdoc, a child method's return type at call sites remains what
the method itself declares (mixed here), not what the parent's docblock says.
The @mir-check below would fail if the child silently inherited Cat.
===config===
suppress=UnusedVariable,UnusedParam,MixedAssignment,MixedArgument
php_version=8.2
===file===
<?php
class Cat {}

abstract class Base {
    /** @return Cat */
    abstract public function make(): mixed;
}

class Child extends Base {
    public function make(): mixed {
        return new Cat();
    }
}

function test(Child $c): void {
    $result = $c->make();
    /** @mir-check $result is mixed */
    echo get_class($result);
}
===expect===

