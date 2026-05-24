===description===
callableInstanceArrayMethodClassContextPhpNativeUnsupportedNonStaticNonPublic
===file===
<?php
class Foo {
    public function __construct() {
        header_register_callback(array($this, "hello"));
    }

    private function hello(): void {
        header("X-Test: hello");
    }
}
===expect===
InvalidArgument
===ignore===
TODO
