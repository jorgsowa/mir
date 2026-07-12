===description===
A final class named only via the class-name string argument of
`method_exists('Foo', 'bar')` must not be reported UnusedClass.
===config===
suppress=
===file===
<?php
final class Foo {
    public static function bar(): void {}
}

method_exists('Foo', 'bar');
===expect===
