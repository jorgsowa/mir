===description===
Wrong case class name in a union type hint is reported; correct-case member is not.
===config===
suppress=UnusedParam
===file===
<?php
class Foo {}
class Bar {}
function process(FOO|Bar $x): void {}
===expect===
WrongCaseClass@4:18-4:21: Class name 'FOO' has incorrect casing; use 'Foo'
