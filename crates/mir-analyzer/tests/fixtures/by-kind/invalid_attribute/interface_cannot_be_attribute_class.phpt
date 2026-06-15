===description===
Interface cannot be attribute class
===file===
<?php
#[Attribute]
interface Foo {}
===expect===
InvalidAttribute@2:2-2:11: Interfaces cannot be attribute classes
