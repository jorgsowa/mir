===description===
Undefined method in [Foo::class, 'method'] callable array
===config===
<mir>
  <issueHandlers>
    <MissingReturnType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Handler {
    public function handle() {
        return "handled";
    }
}

/**
 * @param callable $callback
 */
function executeCallback($callback) {
    return $callback();
}

executeCallback([Handler::class, "nonExistentMethod"]);
//              ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ UndefinedMethod: Method Handler::nonExistentMethod() does not exist
