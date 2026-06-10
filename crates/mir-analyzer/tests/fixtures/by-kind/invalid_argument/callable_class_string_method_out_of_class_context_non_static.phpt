===description===
Callable class string method out of class context non static
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
    public function hello(): void {
        echo "hello";
    }
}

run("Foo::hello");
===expect===
