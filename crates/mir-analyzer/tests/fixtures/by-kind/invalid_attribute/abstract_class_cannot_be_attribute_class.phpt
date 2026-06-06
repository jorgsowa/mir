===description===
Abstract class cannot be attribute class
===file===
<?php
#[Attribute]
abstract class Baz {}
===expect===
InvalidAttribute@2:3-2:12: Abstract classes cannot be attribute classes
