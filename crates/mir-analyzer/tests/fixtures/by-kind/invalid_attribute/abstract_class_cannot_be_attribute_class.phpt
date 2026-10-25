===description===
Abstract class cannot be attribute class
===file===
<?php
#[Attribute]
//^^^^^^^^^ CompileError: Cannot apply #[\Attribute] to abstract class Baz [attribute-on-invalid-class]
abstract class Baz {}
