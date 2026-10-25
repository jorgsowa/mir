===description===
Attribute invalid target property
===file===
<?php
class Foo {
    #[Attribute]
//    ^^^^^^^^^ CompileError: Attribute "Attribute" cannot target property (allowed targets: class) [attribute-target-mismatch]
    public string $bar = "baz";
}
