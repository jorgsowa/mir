===description===
A private static method called only through `$cls::method()` where `$cls` holds a class-string variable must not be reported unused.
===config===
suppress=
===file===
<?php
class Foo {
    private static function helper(): int {
        return 1;
    }

    public static function get(): int {
        $cls = self::class;
        return $cls::helper();
    }
}

Foo::get();
===expect===
