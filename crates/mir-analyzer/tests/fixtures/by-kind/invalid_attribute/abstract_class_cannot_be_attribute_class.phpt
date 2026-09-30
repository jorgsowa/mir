===description===
Abstract class cannot be attribute class
===file===
<?php
#[Attribute]
//^^^^^^^^^ InvalidAttribute: Abstract classes cannot be attribute classes
abstract class Baz {}
===expect===
