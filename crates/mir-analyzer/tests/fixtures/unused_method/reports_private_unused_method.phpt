===source===
<?php
class Foo {
    private function helper(): void {}
}
===expect===
UnusedMethod: Private method Foo::helper() is never called
