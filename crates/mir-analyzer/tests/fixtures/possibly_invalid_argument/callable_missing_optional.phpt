===description===
callableMissingOptional
===file===
<?php
                    /**
                     * @param callable(string=):bool $arg
                     * @return void
                     */
                    function foo($arg) {}

                    function bar(): bool {
                        return rand(0, 10) > 5 ? true : false;
                    }

                    foo("bar");
===expect===
PossiblyInvalidArgument
===ignore===
TODO
