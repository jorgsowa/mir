===description===
Undefined method in [Foo::class, 'method'] callable array
===config===
suppress=MissingReturnType
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
===expect===
UndefinedMethod@15:16-15:53: Method Handler::nonExistentMethod() does not exist
