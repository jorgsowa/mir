===description===
Static invocation
===file===
<?php
class Foo {
    public function barBar(): void {}
}

Foo::barBar();
===expect===
InvalidStaticInvocation
===ignore===
TODO
