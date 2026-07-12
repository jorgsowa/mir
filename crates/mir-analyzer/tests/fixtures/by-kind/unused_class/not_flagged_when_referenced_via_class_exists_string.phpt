===description===
A final class named only in a `class_exists('Foo')` string-literal check
must not be reported UnusedClass.
===config===
suppress=
===file===
<?php
final class Foo {}

if (!class_exists('Foo')) {
}
===expect===
