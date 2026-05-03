===description===
invalidArrayKeyType
===file===
<?php
                    /**
                     * @param array<float, string> $arg
                     * @return void
                     */
                    function foo($arg) {}
===expect===
InvalidDocblock
===ignore===
TODO
