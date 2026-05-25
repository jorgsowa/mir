===description===
Trait cannot be attribute class
===file===
<?php
#[Attribute]
trait Foo {}
===expect===
InvalidAttribute
===ignore===
TODO
