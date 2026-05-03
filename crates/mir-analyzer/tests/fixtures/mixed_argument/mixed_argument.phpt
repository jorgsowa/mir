===description===
mixedArgument
===file===
<?php
                    function fooFoo(int $a): void {}
                    /** @var mixed */
                    $a = "hello";
                    fooFoo($a);
===expect===
MixedArgument
===ignore===
TODO
