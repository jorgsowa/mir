===description===
A class named only in a `class_alias('Foo', 'Bar')` string-literal call
must not be reported UnusedClass.
===config===
suppress=
===file===
<?php
final class Foo {}

class_alias('Foo', 'Bar');
===expect===
