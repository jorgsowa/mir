===description===
A class named only in a `class_implements('Foo')` string-literal call
must not be reported UnusedClass.
===config===
suppress=
===file===
<?php
interface Bar {}
final class Foo implements Bar {}

class_implements('Foo');
===expect===
