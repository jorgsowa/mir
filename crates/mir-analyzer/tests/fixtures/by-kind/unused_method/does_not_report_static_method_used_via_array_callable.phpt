===description===
A private static method used only through the `['Foo', 'helper']`
class-string array-callable literal must not be reported unused.
===config===
suppress=UnusedVariable
===file===
<?php
class Foo {
    private static function helper(): void {}

    public static function run(): void {
        $c = ['Foo', 'helper'];
        call_user_func($c);
    }
}
===expect===
