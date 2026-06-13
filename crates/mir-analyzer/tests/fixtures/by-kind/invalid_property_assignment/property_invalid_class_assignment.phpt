===description===
Property invalid class assignment
===file===
<?php
namespace Bar;

class PropertyType {}
class SomeOtherPropertyType {}

/**
 * @property PropertyType $foo
 */
class A {
    /** @param string $name */
    public function __get($name): ?string {
        if ($name === "foo") {
            return "hello";
        }

        return null;
    }

    /**
     * @param string $name
     * @param mixed $value
     */
    public function __set($name, $value): void {
    }
}

$a = new A();
$a->foo = new SomeOtherPropertyType();
===expect===
MissingConstructor@10:0-10:9: Class Bar\A has uninitialized properties but no constructor
InvalidPropertyAssignment@29:1-29:38: Property $foo expects 'PropertyType', cannot assign 'Bar\SomeOtherPropertyType'
