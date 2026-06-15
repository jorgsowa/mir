===description===
Multiple attributes show errors
===file===
<?php
#[Attribute(Attribute::TARGET_CLASS)]
class Foo {}

#[Attribute(Attribute::TARGET_PARAMETER)]
class Bar {}

#[Foo, Bar]
class Baz {}

===expect===
InvalidAttribute@8:7-8:10: Attribute Bar cannot be used on this target
