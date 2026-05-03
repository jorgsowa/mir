===description===
wrongArg
===file===
<?php
                    $bar = ["foo", "bar"];

                    $bam = array_map(
                        function(int $a): int {
                            return $a + 1;
                        },
                        $bar
                    );
===expect===
InvalidScalarArgument
===ignore===
TODO
