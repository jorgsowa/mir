===description===
Trait cannot be attribute class
===file===
<?php
#[Attribute]
trait Foo {}
===expect===
InvalidAttribute@2:2-2:11: Traits cannot be attribute classes
