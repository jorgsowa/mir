===description===
UndefinedAttributeClass does NOT fire for a properly defined attribute class.
===file===
<?php
#[\Attribute]
class MyAttr {}

#[MyAttr]
class Foo {}
===expect===
