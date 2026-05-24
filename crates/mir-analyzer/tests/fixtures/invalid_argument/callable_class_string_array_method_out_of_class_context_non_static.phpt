===description===
callableClassStringArrayMethodOutOfClassContextNonStatic
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

run(array(Foo::class, "hello"));
===expect===
InvalidArgument
===ignore===
TODO
