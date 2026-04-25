===config===
find_dead_code=true
===file===
<?php
class Foo {
    private function helper(): void {}
}
===expect===
UnusedMethod: Private method Foo::helper() is never called
