===description===
checkMixedMethodCallStaticMethodCallArg
===file===
<?php
                    class B {}
                    /** @param mixed $a */
                    function foo($a) : void {
                        /** @psalm-suppress MixedMethodCall */
                        $a->bar(B::bat());
                    }
===expect===
UndefinedMethod
===ignore===
TODO
