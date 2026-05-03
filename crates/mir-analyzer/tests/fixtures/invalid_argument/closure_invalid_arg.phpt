===description===
closureInvalidArg
===file===
<?php
                    /** @param Closure(int): string $c */
                    function takesClosure(Closure $c): void {}

                    takesClosure(5);
===expect===
InvalidArgument
===ignore===
TODO
