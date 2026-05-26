===description===
Repeat non repeatable attribute
===file===
<?php
#[Attribute]
class Foo {}

#[Foo, Foo]
class Baz {}

===expect===
InvalidAttribute
===ignore===
TODO
