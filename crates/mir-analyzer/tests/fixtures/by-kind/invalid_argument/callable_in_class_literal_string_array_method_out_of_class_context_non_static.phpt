===description===
Callable in class literal string array method out of class context non static
===ignore===
TODO
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
        run(array("Foo", "hello"));
    }

    public function hello(): void {
        echo "hello";
    }
}
===expect===
