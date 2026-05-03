===description===
arrayReduceInvalidClosureTooFewArgs
===file===
<?php
                    $arr = [2, 3, 4, 5];

                    $direct_closure_result = array_reduce(
                        $arr,
                        function() : int {
                            return 5;
                        },
                        1
                    );
===expect===
InvalidArgument
===ignore===
TODO
