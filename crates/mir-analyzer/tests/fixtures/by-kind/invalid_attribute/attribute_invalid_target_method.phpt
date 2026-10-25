===description===
Attribute invalid target method
===file===
<?php
class Foo {
    #[Attribute]
//    ^^^^^^^^^ CompileError: Attribute "Attribute" cannot target method (allowed targets: class) [attribute-target-mismatch]
    public function bar(): void {}
}
