===description===
Interface cannot be attribute class
===file===
<?php
#[Attribute]
//^^^^^^^^^ CompileError: Cannot apply #[\Attribute] to interface Foo [attribute-on-invalid-class]
interface Foo {}
