===description===
Wrong case in attribute class name is reported.
===file===
<?php
#[\Attribute]
class myAttr {}

#[myattr]
class Foo {}
===expect===
WrongCaseClass@5:3-5:9: Class name 'myattr' has incorrect casing; use 'myAttr'

