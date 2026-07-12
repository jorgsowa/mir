===description===
A private static method used only through the `call_user_func('Foo::helper')`
class-string-with-method callable form must not be reported unused.
===config===
suppress=
===file===
<?php
class Foo {
    private static function helper(): void {}

    public static function run(): void {
        call_user_func('Foo::helper');
    }
}
===expect===
