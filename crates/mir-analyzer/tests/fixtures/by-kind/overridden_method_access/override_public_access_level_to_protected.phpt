===description===
Override public access level to protected
===file===
<?php
class A {
    public function fooFoo(): void {}
}

class B extends A {
    protected function fooFoo(): void {}
}
===expect===
OverriddenMethodAccess@7:4-7:40: Method B::foofoo() overrides with less visibility
