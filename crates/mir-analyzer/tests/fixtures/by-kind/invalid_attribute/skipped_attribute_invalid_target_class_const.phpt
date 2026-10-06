===description===
SKIPPED-attributeInvalidTargetClassConst
===file===
<?php
class Foo {
    #[Attribute]
//    ^^^^^^^^^ InvalidAttribute: #[Attribute] can only be applied to classes, not constants
    public const BAR = "baz";
}
