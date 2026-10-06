===description===
$this::undefinedMethod() should still emit UndefinedMethod
===file===
<?php
class Foo {
    public function test(): void {
        $this::nonExistent();
//      ^^^^^^^^^^^^^^^^^^^^ UndefinedMethod: Method Foo::nonExistent() does not exist
    }
}
