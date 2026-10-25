===description===
SKIPPED-attributeInvalidTargetClassConst
===file===
<?php
class Foo {
    #[Attribute]
//    ^^^^^^^^^ CompileError: Attribute "Attribute" cannot target class constant (allowed targets: class) [attribute-target-mismatch]
    public const BAR = "baz";
}
