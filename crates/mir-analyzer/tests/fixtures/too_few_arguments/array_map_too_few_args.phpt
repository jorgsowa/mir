===description===
arrayMapTooFewArgs
===file===
<?php
                    function foo(int $i, string $s) : bool {
                      return true;
                    }

                    array_map("foo", [1, 2, 3]);
===expect===
UnusedParam@2:33: Parameter $i is never used
UnusedParam@2:41: Parameter $s is never used
TooFewArguments@6:30: Too few arguments for foo(): expected 2, got 1
