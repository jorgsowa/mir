===description===
Abstract class cannot be attribute class
===file===
<?php
#[Attribute]
abstract class Baz {}
===expect===
InvalidAttribute@2:2-2:11: Abstract classes cannot be attribute classes
