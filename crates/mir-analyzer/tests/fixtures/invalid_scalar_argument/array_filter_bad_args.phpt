===description===
arrayFilterBadArgs
===file===
<?php
                    function foo(int $i) : bool {
                      return true;
                    }

                    array_filter(["hello"], "foo");
===expect===
InvalidScalarArgument
===ignore===
TODO
