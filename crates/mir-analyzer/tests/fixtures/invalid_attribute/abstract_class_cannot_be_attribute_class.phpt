===description===
Abstract class cannot be attribute class
===file===
<?php
#[Attribute]
abstract class Baz {}
===expect===
InvalidAttribute
===ignore===
TODO
