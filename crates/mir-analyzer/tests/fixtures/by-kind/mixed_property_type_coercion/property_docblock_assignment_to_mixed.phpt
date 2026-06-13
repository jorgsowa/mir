===description===
Property docblock assignment to mixed
===file===
<?php
/**
 * @property string $foo
 */
class A {
    public function __get(string $name): ?string {
        if ($name === "foo") {
            return "hello";
        }

        return null;
    }

    /** @param mixed $value */
    public function __set(string $name, $value): void {
    }
}

/** @param mixed $b */
function foo($b) : void {
    $a = new A();
    $a->__set("foo", $b);
}
===expect===
MissingConstructor@5:0-5:9: Class A has uninitialized properties but no constructor
