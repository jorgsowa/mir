===description===
Attribute target arg cannot be variable
===ignore===
TODO
===file===
<?php
$target = 1;

#[Attribute($target)]
class Foo {}

===expect===
