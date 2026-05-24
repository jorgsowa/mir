===description===
repeatNonRepeatableAttribute
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
