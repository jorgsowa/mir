===description===
propertyDocblockAssignmentToMixed
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
MixedPropertyTypeCoercion
===ignore===
TODO
