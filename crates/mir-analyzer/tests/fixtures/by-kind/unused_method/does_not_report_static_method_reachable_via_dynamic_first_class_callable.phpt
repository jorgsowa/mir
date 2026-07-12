===description===
A private static method reachable only via a dynamic static
first-class-callable (self::$name(...)) must not be reported unused.
===config===
suppress=
===file===
<?php
class Foo {
    private static function helper(): void {}

    public static function run(string $name): callable {
        return self::$name(...);
    }
}
===expect===
