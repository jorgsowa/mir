===description===
`@psalm-assert-if-true`/`@psalm-assert-if-false` narrows a static-property
argument like its var/instance-property counterparts already do.
===config===
suppress=MissingConstructor,UnusedParam,MissingParamType,MissingPropertyType
===file===
<?php
class Foo {
    public function bar(): void {}
}

/** @psalm-assert-if-true Foo $value */
function isFoo($value): bool {
    return $value instanceof Foo;
}

/** @psalm-assert-if-false Foo $value */
function isNotFoo($value): bool {
    return !($value instanceof Foo);
}

class Container {
    private static $prop = null;

    public static function testAssertIfTrue(): void {
        if (isFoo(self::$prop)) {
            self::$prop->bar();
        }
    }

    public static function testAssertIfFalse(): void {
        if (!isNotFoo(self::$prop)) {
            self::$prop->bar();
        }
    }
}
===expect===
