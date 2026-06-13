===description===
Correct case class name in new expression is not reported.
===config===
suppress=UnusedVariable
===file===
<?php
class Foo {}
$x = new Foo();
===expect===
