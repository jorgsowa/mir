===description===
Interface cannot be attribute class
===file===
<?php
#[Attribute]
interface Foo {}
===expect===
InvalidAttribute
===ignore===
TODO
