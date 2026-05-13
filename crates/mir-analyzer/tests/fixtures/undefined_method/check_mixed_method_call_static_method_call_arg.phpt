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
MixedMethodCall@6:24: Method bar() called on mixed type
UndefinedMethod@6:32: Method B::bat() does not exist
