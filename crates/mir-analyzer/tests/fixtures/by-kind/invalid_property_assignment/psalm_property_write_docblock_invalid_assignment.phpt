===description===
Psalm property write docblock invalid assignment
===file===
<?php
/**
 * @psalm-property-write string $foo
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
$a->foo = 5;
===expect===
InvalidPropertyAssignment@20:1-20:12: Property $foo expects 'string', cannot assign '5'
