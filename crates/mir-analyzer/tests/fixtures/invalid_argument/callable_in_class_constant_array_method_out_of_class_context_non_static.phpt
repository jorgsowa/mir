===description===
callableInClassConstantArrayMethodOutOfClassContextNonStatic
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
                        run(array(__CLASS__, "hello"));
                    }

                    public function hello(): void {
                        echo "hello";
                    }
                }
===expect===
InvalidArgument
===ignore===
TODO
