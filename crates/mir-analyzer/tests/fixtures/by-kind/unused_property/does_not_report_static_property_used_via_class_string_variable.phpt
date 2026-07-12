===description===
A private static property read only through `$cls::$prop` where `$cls` holds a class-string variable must not be reported unused.
===config===
suppress=
===file===
<?php
class Foo {
    private static int $count = 0;

    public static function get(): int {
        $cls = self::class;
        return $cls::$count;
    }
}

Foo::get();
===expect===
