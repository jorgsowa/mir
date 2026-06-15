===description===
Wrong case in attribute class name is reported.
===file===
<?php
#[\Attribute]
class myAttr {}

#[myattr]
class Foo {}
===expect===
WrongCaseClass@5:2-5:8: Class name 'myattr' has incorrect casing; use 'myAttr'
