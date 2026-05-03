===description===
invalidArgumentFalseTrueExpected
===file===
<?php
                    /**
                     * @param true|string $arg
                     * @return void
                     */
                    function foo($arg) {}

                    foo(false);
===expect===
InvalidArgument
===ignore===
TODO
