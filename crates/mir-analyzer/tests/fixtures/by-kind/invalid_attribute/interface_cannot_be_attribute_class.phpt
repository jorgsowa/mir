===description===
Interface cannot be attribute class
===file===
<?php
#[Attribute]
interface Foo {}
===expect===
InvalidAttribute@2:3-2:12: Interfaces cannot be attribute classes
