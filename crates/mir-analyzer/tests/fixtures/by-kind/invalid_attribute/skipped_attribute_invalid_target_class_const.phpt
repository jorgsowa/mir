===description===
SKIPPED-attributeInvalidTargetClassConst
===file===
<?php
class Foo {
    #[Attribute]
    public const BAR = "baz";
}

===expect===
InvalidAttribute
===ignore===
TODO
