===description===
Attribute class has no attribute annotation
===file===
<?php
class A {}

#[A]
class B {}
===expect===
InvalidAttribute@4:2-4:3: Class A does not have an #[Attribute] annotation
