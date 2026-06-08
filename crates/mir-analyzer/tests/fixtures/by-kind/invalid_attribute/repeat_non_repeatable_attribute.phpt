===description===
Repeat non repeatable attribute
===file===
<?php
#[Attribute]
class Foo {}

#[Foo, Foo]
class Baz {}

===expect===
InvalidAttribute@5:3-5:3: Attribute Foo is not repeatable
InvalidAttribute@5:8-5:11: Attribute Foo is not repeatable
