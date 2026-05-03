===description===
callableClassStringArrayMethodOtherClassContextNonStaticPublic
===file===
<?php
                class Foo {
                    public function __construct() {
                        $bar = new Bar();
                        $bar->run_in_c(array(Foo::class, "hello"));
                    }

                    public function hello(): void {
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
