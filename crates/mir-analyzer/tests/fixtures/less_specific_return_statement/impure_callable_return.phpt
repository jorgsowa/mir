===description===
impureCallableReturn
===file===
<?php
                    /**
                     * @psalm-pure
                     * @return pure-callable():int
                     */
                    function foo(): callable {
                        /** @psalm-suppress ImpureFunctionCall */
                        return function() {
                            echo "bar";
                            return 1;
                        };
                    }
===expect===
LessSpecificReturnStatement
===ignore===
TODO
