===description===
Trait cannot be attribute class
===file===
<?php
#[Attribute]
//^^^^^^^^^ InvalidAttribute: Traits cannot be attribute classes
trait Foo {}
