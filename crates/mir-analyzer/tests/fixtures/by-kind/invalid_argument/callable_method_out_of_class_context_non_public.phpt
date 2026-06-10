===description===
Callable method out of class context non public
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
    private static function hello(): void {
        echo "hello";
    }
}

$foo = new Foo();
run(array($foo, "hello"));
===expect===
