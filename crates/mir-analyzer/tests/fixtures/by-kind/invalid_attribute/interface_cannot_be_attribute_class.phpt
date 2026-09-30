===description===
Interface cannot be attribute class
===file===
<?php
#[Attribute]
//^^^^^^^^^ InvalidAttribute: Interfaces cannot be attribute classes
interface Foo {}
===expect===
