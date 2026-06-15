===description===
SKIPPED-attributeInvalidTargetClassConst
===file===
<?php
class Foo {
    #[Attribute]
    public const BAR = "baz";
}

===expect===
InvalidAttribute@3:6-3:15: #[Attribute] can only be applied to classes, not constants
