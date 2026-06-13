===description===
Attribute target arg cannot be variable
===config===
suppress=UnusedVariable
===file===
<?php
$target = 1;

#[Attribute($target)]
class Foo {}

===expect===
