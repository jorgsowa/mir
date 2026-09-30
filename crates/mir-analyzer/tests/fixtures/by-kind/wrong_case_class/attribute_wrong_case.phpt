===description===
Wrong case in attribute class name is reported.
===file===
<?php
#[\Attribute]
class myAttr {}

#[myattr]
//^^^^^^ WrongCaseClass: Class name 'myattr' has incorrect casing; use 'myAttr'
class Foo {}
===expect===
