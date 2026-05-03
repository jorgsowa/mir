===description===
arrayMapTooManyArgs
===file===
<?php
                    function foo() : bool {
                      return true;
                    }

                    array_map("foo", [1, 2, 3]);
===expect===
TooManyArguments
===ignore===
TODO
