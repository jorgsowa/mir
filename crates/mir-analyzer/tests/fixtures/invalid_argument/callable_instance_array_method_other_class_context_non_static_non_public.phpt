===description===
callableInstanceArrayMethodOtherClassContextNonStaticNonPublic
===file===
<?php
class Foo {
    public function __construct() {
        $bar = new Bar();
        $bar->run_in_c(array($this, "hello"));
    }

    protected function hello(): void {
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
InvalidArgument
===ignore===
TODO
