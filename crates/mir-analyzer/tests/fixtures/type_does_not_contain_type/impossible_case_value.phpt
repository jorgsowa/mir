===description===
impossibleCaseValue
===file===
<?php
                    $a = rand(0, 1) ? "a" : "b";

                    switch ($a) {
                        case "a":
                            break;

                        case "b":
                            break;

                        case "c":
                            echo "impossible";
                    }
===expect===
TypeDoesNotContainType
===ignore===
TODO
