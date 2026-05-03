===description===
enumStringOrEnumIntIncorrectString
===file===
<?php
                    namespace Ns;

                    /** @psalm-param ( "foo" | "bar" | 1 | 2 | 3 ) $s */
                    function foo($s) : void {}
                    foo("bat");
===expect===
InvalidArgument
===ignore===
TODO
