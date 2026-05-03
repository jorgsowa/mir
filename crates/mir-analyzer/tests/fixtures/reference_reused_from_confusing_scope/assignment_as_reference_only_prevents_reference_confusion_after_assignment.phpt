===description===
assignmentAsReferenceOnlyPreventsReferenceConfusionAfterAssignment
===file===
<?php
                    $arr = [1, 2, 3];
                    foreach ($arr as &$i) {
                        ++$i;
                    }

                    for ($i = 0; $i < 10; ++$i) {
                        echo $i;
                    }

                    $i = &$foo;
                
===expect===
ReferenceReusedFromConfusingScope
===ignore===
TODO
