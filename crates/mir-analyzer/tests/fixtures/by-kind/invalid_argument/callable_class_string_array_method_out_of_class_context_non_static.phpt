===description===
Callable class string array method out of class context non static
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
