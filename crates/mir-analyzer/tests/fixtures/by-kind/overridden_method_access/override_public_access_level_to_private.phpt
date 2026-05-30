===description===
Override public access level to private
===file===
<?php
class A {
    public function fooFoo(): void {}
}

class B extends A {
    private function fooFoo(): void {}
}
===expect===
OverriddenMethodAccess@7:4-7:38: Method B::foofoo() overrides with less visibility
