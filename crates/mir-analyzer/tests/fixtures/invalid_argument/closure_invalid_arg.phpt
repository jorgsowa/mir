===description===
closureInvalidArg
===file===
<?php
                    /** @param Closure(int): string $c */
                    function takesClosure(Closure $c): void {}

                    takesClosure(5);
===expect===
UnusedParam@3:42: Parameter $c is never used
InvalidArgument@5:33: Argument $c of takesClosure() expects 'Closure(int): string', got '5'
