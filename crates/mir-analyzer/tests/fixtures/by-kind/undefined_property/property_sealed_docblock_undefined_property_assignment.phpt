===description===
Property sealed docblock undefined property assignment
===file===
<?php
/**
 * @property string $foo
 * @seal-properties
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

$a = new A();
$a->bar = 5;
===expect===
MissingConstructor@6:0-6:9: Class A has uninitialized properties but no constructor
