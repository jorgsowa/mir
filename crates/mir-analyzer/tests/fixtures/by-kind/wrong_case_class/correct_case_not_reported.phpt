===description===
Correct case class name in new expression is not reported.
===file===
<?php
class Foo {}
$x = new Foo();
===expect===
