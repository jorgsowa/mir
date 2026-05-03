===description===
invalidArgumentCallableWithoutArgsUnion
===file===
<?php
                    function foo(int $a): void {}

                    /**
                     * @param callable()|float $callable
                     * @return void
                     */
                    function acme($callable) {}
                    acme("foo");
===expect===
InvalidArgument
===ignore===
TODO
