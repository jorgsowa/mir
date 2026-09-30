===description===
Attribute invalid target method
===file===
<?php
class Foo {
    #[Attribute]
//    ^^^^^^^^^ InvalidAttribute: #[Attribute] can only be applied to classes, not methods
    public function bar(): void {}
}

===expect===
