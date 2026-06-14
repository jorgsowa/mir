===description===
Correct case in attribute class name is not reported.
===file===
<?php
#[\Attribute]
class MyAttr {}

#[MyAttr]
class Foo {}
===expect===
