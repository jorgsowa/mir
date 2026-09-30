===description===
Static invocation
===file===
<?php
class Foo {
    public function barBar(): void {}
}

Foo::barBar();
//<^^^^^^^^^^^^^ InvalidStaticInvocation: Non-static method Foo::barBar() cannot be called statically
===expect===
