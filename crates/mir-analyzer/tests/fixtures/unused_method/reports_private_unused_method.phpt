===description===
reports private unused method
===config===
find_dead_code=true
===file===
<?php
class Foo {
    private function helper(): void {}
}
===expect===
UnusedMethod@1:0: Private method Foo::helper() is never called
===ignore===
TODO
