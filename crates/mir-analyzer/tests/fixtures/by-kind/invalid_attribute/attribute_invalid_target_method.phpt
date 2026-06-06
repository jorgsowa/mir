===description===
Attribute invalid target method
===file===
<?php
class Foo {
    #[Attribute]
    public function bar(): void {}
}

===expect===
InvalidAttribute@3:7-3:16: #[Attribute] can only be applied to classes, not methods
