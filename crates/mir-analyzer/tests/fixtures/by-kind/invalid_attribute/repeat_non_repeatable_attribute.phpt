===description===
Repeat non repeatable attribute
===file===
<?php
#[Attribute]
class Foo {}

#[Foo, Foo]
//^^^ InvalidAttribute: Attribute Foo is not repeatable
//     ^^^ InvalidAttribute: Attribute Foo is not repeatable
class Baz {}

===expect===
