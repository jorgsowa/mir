===description===
Attribute class has no attribute annotation
===file===
<?php
class A {}

#[A]
class B {}
===expect===
InvalidAttribute@4:3-4:4: Class A does not have an #[Attribute] annotation
