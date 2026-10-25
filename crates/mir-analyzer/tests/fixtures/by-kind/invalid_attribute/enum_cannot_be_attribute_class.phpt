===description===
Enum cannot be attribute class
===file===
<?php
#[Attribute]
//^^^^^^^^^ CompileError: Cannot apply #[\Attribute] to enum Foo [attribute-on-invalid-class]
enum Foo {
    case Bar;
}
