===description===
A private static method used only through first-class-callable syntax
(`self::helper(...)`) must not be reported unused.
===config===
suppress=UnusedVariable
===file===
<?php
class Foo {
    private static function helper(): void {}

    public static function run(): void {
        $c = self::helper(...);
        $c();
    }
}
===expect===
