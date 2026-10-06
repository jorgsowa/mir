===description===
reports private unused method
===file===
<?php
class Foo {
    private function helper(): void {}
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ UnusedMethod: Private method Foo::helper() is never called
}
