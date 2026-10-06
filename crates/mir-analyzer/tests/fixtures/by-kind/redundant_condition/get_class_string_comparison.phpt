===description===
get_class() string comparison narrowing
===config===
<mir>
  <issueHandlers>
    <MissingReturnType errorLevel="suppress"/>
  </issueHandlers>
</mir>
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
//            ^^^^^^^^^^^^^^^^^^^^^^^^^ RedundantCondition: Condition is always true, so the check is redundant
        $obj->bar();
    }
}
