===description===
Unguarded array callables report undefined methods.
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
    register_shutdown([Other::class, 'gateStatic']);
//                    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^ UndefinedMethod: Method Other::gateStatic() does not exist
}
