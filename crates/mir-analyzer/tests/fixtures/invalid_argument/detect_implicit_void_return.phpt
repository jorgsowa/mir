===description===
detectImplicitVoidReturn
===file===
<?php
                    /**
                     * @param Closure():Exception $c
                     */
                    function takesClosureReturningException(Closure $c) : void {
                        echo $c()->getMessage();
                    }

                    takesClosureReturningException(
                        function () {
                            echo "hello";
                        }
                    );
===expect===
InvalidArgument
===ignore===
TODO
