===description===
A class named only in a `class_uses('Foo')` string-literal call
must not be reported UnusedClass.
===config===
suppress=
===file===
<?php
trait Helper {}
final class Foo {
    use Helper;
}

class_uses('Foo');
===expect===
