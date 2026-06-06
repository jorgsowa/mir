===description===
Trait cannot be attribute class
===file===
<?php
#[Attribute]
trait Foo {}
===expect===
InvalidAttribute@2:3-2:12: Traits cannot be attribute classes
