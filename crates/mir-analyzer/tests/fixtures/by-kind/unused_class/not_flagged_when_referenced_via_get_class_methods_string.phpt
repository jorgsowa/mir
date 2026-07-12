===description===
A class named only in a `get_class_methods('Foo')` string-literal call
must not be reported UnusedClass.
===config===
suppress=
===file===
<?php
final class Foo {
    public function bar(): void {}
}

get_class_methods('Foo');
===expect===
