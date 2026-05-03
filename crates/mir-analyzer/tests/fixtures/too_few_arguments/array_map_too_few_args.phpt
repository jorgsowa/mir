===description===
arrayMapTooFewArgs
===file===
<?php
                    function foo(int $i, string $s) : bool {
                      return true;
                    }

                    array_map("foo", [1, 2, 3]);
===expect===
TooFewArguments
===ignore===
TODO
