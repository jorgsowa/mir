===description===
callableWithTooManyArgs
===file===
<?php
                    /**
                     * @param callable(string):void $fn
                     */
                    function test(callable $fn): void {
                        $fn('hello', 'world');
                    }

===expect===
TooManyArguments
===ignore===
TODO
