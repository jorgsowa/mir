===description===
Static invocation
===file===
<?php
class Foo {
    public function barBar(): void {}
}

Foo::barBar();
===expect===
InvalidStaticInvocation@6:1-6:14: Non-static method Foo::barBar() cannot be called statically
