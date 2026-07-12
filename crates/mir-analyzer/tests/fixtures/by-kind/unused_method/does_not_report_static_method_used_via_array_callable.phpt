===description===
A private static method used only through the `['Foo', 'helper']`
class-string array-callable literal must not be reported unused.
===config===
suppress=
===file===
<?php
class Foo {
    private static function helper(): void {}

    public static function run(): void {
        call_user_func(['Foo', 'helper']);
    }
}
===expect===
