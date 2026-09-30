===description===
Attribute class has no attribute annotation
===file===
<?php
class A {}

#[A]
//^ InvalidAttribute: Class A does not have an #[Attribute] annotation
class B {}
===expect===
