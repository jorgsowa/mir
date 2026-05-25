===description===
Attribute class has no attribute annotation
===file===
<?php
class A {}

#[A]
class B {}
===expect===
InvalidAttribute
===ignore===
TODO
