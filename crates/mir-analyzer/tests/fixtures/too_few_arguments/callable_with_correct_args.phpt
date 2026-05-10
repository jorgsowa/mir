===description===
callableWithCorrectArgs
===file===
<?php
                    /**
                     * @param callable(string):void $fn
                     */
                    function test(callable $fn): void {
                        $fn('hello');
                    }

===expect===
