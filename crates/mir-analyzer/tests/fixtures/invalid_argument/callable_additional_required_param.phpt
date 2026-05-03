===description===
callableAdditionalRequiredParam
===file===
<?php
                    /**
                     * @param callable(string, string, string):bool $arg
                     * @return void
                     */
                    function foo($arg) {}

                    function bar(string $a, string $b, string $c, string $d): bool {}

                    foo("bar");
===expect===
InvalidArgument
===ignore===
TODO
