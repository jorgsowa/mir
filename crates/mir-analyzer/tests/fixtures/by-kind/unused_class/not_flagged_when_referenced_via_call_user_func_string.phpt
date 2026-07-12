===description===
A final class referenced only by name in a `call_user_func('Foo::helper')`
class-string callable must not be reported UnusedClass.
===config===
suppress=
===file===
<?php
final class Foo {
    public static function helper(): void {}
}

call_user_func('Foo::helper');
===expect===
