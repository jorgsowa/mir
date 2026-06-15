===description===
Repeat non repeatable attribute
===file===
<?php
#[Attribute]
class Foo {}

#[Foo, Foo]
class Baz {}

===expect===
InvalidAttribute@5:2-5:2: Attribute Foo is not repeatable
InvalidAttribute@5:7-5:10: Attribute Foo is not repeatable
