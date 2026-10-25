===description===
Trait cannot be attribute class
===file===
<?php
#[Attribute]
//^^^^^^^^^ CompileError: Cannot apply #[\Attribute] to trait Foo [attribute-on-invalid-class]
trait Foo {}
