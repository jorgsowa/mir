===description===
callableClassStringMethodOutOfClassContextNonPublic
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

run("Foo::hello");
===expect===
InvalidArgument
===ignore===
TODO
