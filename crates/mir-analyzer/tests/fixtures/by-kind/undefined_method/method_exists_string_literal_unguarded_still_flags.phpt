===description===
Without a method_exists() guard, a 'Foo::method' string callable whose
target method does not exist is checked normally and UndefinedMethod is
reported.
===config===
<mir>
  <issueHandlers>
    <MissingReturnType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Other {}

function register_shutdown(callable $cb): void {
    $cb();
}

function free(): void {
    register_shutdown('Other::gateStatic');
//                    ^^^^^^^^^^^^^^^^^^^ UndefinedMethod: Method Other::gateStatic() does not exist
}
===expect===
