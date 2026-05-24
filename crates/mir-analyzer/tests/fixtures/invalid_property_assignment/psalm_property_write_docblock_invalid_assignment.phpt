===description===
psalmPropertyWriteDocblockInvalidAssignment
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
InvalidPropertyAssignmentValue
===ignore===
TODO
