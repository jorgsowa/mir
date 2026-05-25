===description===
Callable in class string method out of class context non public
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
        run("Foo::hello");
    }

    private static function hello(): void {
        echo "hello";
    }
}
===expect===
InvalidArgument
===ignore===
TODO
