===description===
reports private unused method
===file===
<?php
class Foo {
//<^ +2:1 UnusedClass: Class Foo is never referenced
    private function helper(): void {}
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ UnusedMethod: Private method Foo::helper() is never called
}
