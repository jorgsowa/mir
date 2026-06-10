===description===
Callable instance array method class context php native unsupported non static non public
===ignore===
TODO
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
