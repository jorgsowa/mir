===description===
Static invocation
===file===
<?php
class Foo {
    public function barBar(): void {}
}

Foo::barBar();
===expect===
InvalidStaticInvocation@6:0-6:13: Non-static method Foo::barBar() cannot be called statically
