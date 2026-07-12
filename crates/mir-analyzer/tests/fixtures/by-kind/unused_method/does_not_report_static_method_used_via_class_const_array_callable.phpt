===description===
A private static method used only through the `[Foo::class, 'helper']`
array-callable literal must not be reported unused. `Foo::class` evaluates to
a class-string type, distinct from the `['Foo', 'helper']` string-literal form.
===config===
suppress=
===file===
<?php
class Foo {
    private static function helper(): void {}

    public static function run(): void {
        call_user_func([Foo::class, 'helper']);
    }
}
===expect===
