===description===
callableWithVariadicParams
===file===
<?php
                    /**
                     * @param callable(string, ...$args):void $fn
                     */
                    function test(callable $fn): void {
                        $fn('hello', 'world', 'extra');
                    }

===expect===
