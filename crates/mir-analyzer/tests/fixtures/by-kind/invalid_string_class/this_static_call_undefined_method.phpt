===description===
$this::undefinedMethod() should still emit UndefinedMethod
===file===
<?php
class Foo {
    public function test(): void {
        $this::nonExistent();
    }
}
===expect===
UndefinedMethod@4:9-4:29: Method Foo::nonExistent() does not exist
