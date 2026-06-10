===description===
Callable in class constant array method out of class context non static
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
        run(array(__CLASS__, "hello"));
    }

    public function hello(): void {
        echo "hello";
    }
}
===expect===
