===description===
Valid method in [Foo::class, 'method'] callable array is not flagged
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

executeCallback([Handler::class, "handle"]);
===expect===
