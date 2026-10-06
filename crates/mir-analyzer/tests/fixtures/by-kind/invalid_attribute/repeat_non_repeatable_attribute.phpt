===description===
Repeat non repeatable attribute
===file===
<?php
#[Attribute]
class Foo {}

#[Foo, Foo]
//     ^^^ InvalidAttribute: Attribute Foo is not repeatable
class Baz {}

===expect===
InvalidAttribute@5:2-5:5: Attribute Foo is not repeatable
