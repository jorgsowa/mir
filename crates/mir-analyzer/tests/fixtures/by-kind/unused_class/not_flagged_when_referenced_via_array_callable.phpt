===description===
A final class referenced only by name in a `['Foo', 'helper']` array-callable
literal must not be reported UnusedClass.
===config===
suppress=
===file===
<?php
final class Foo {
    public static function helper(): void {}
}

call_user_func(['Foo', 'helper']);
===expect===
