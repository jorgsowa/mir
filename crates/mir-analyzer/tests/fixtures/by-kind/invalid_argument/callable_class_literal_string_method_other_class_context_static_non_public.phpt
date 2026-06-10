===description===
Callable class literal string method other class context static non public
===file===
<?php
class Foo {
    public function __construct() {
        $bar = new Bar();
        $bar->run_in_c("Foo::hello");
    }

    protected static function hello(): void {
        echo "hello";
    }
}

class Bar {
    /**
     * @param callable $callable
     * @return void
     */
    public function run_in_c($callable) {
        call_user_func($callable);
    }
}
===expect===
