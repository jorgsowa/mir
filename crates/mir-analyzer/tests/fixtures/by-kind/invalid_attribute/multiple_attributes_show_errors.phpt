===description===
Multiple attributes show errors
===file===
<?php
#[Attribute(Attribute::TARGET_CLASS)]
class Foo {}

#[Attribute(Attribute::TARGET_PARAMETER)]
class Bar {}

#[Foo, Bar]
//     ^^^ InvalidAttribute: Attribute Bar cannot be used on this target
class Baz {}

===expect===
