===description===
Callable class string method out of class context non static non public
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
    protected function hello(): void {
        echo "hello";
    }
}

run("Foo::hello");
===expect===
InvalidArgument
===ignore===
TODO
