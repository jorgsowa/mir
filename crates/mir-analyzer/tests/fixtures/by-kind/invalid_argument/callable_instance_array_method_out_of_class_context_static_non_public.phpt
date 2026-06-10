===description===
Callable instance array method out of class context static non public
===file===
<?php
/**
 * @param callable $callable
 * @return void
 */
function run($callable) {
    call_user_func($callable);
}

class Foo {
    public function __construct() {
        run(array($this, "hello"));
    }

    protected static function hello(): void {
        echo "hello";
    }
}
===expect===
