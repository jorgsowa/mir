===description===
Magic setter invalid assignment type
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

    public function badSet(): void {
        $this->__set("foo", new stdClass());
    }
}
===expect===
MissingConstructor@5:0-5:9: Class A has uninitialized properties but no constructor
