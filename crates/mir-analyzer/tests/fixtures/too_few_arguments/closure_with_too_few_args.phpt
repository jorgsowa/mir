===description===
closureWithTooFewArgs
===file===
<?php
                    /**
                     * @param Closure(string, int):void $fn
                     */
                    function test(callable $fn): void {
                        $fn('hello');
                    }

===expect===
TooFewArguments
===ignore===
TODO
