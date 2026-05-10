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
TooFewArguments@6:24: Too few arguments for callable(): expected 2, got 1
