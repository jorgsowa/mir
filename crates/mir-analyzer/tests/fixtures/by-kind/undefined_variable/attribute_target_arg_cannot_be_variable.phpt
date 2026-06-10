===description===
Attribute target arg cannot be variable
===file===
<?php
$target = 1;

#[Attribute($target)]
class Foo {}

===expect===
