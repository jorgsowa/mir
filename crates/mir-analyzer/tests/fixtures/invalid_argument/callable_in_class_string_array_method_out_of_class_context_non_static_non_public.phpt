===description===
Callable in class string array method out of class context non static non public
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

    protected function hello(): void {
        echo "hello";
    }
}
===expect===
InvalidArgument
===ignore===
TODO
