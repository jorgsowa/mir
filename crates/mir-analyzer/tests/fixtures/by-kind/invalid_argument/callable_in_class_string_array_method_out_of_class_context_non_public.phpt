===description===
Callable in class string array method out of class context non public
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
        run(array(Foo::class, "hello"));
    }

    private static function hello(): void {
        echo "hello";
    }
}
===expect===
