===description===
get_class() string comparison narrowing
===config===
suppress=MissingReturnType
===file===
<?php
final class Foo {
    public function foo() {}
}

final class Bar {
    public function bar() {}
}

function testGetClassSimple(object $obj) {
    if (get_class($obj) === 'Foo') {
        $obj->foo();
    }
}

function testGetClassElseif(Foo|Bar $obj) {
    if (get_class($obj) === 'Foo') {
        $obj->foo();
    } elseif (get_class($obj) === 'Bar') {
        $obj->bar();
    }
}
===expect===
RedundantCondition@19:14-19:39: Condition is always true/false for type 'bool'
