===description===
Property sealed docblock undefined property fetch
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
echo $a->bar;
===expect===
MissingConstructor@6:0-6:9: Class A has uninitialized properties but no constructor
