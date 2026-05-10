===description===
callableNoArgsNeeded
===file===
<?php
                    /**
                     * @param callable():void $fn
                     */
                    function test(callable $fn): void {
                        $fn();
                    }

===expect===
