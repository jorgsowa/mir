===description===
detectUnusedVariableInsideLoopCalledInFunction
===file===
<?php
                    function foo(int $s) : int {
                        return $s;
                    }

                    function bar() : void {
                        foreach ([1, 2, 3] as $i) {
                            $i = foo($i);
                        }
                    }
===expect===
UnusedVariable
===ignore===
TODO
