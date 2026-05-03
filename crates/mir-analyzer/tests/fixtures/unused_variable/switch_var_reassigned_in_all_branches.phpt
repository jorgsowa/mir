===description===
switchVarReassignedInAllBranches
===file===
<?php
                    $a = false;

                    switch (rand(0, 2)) {
                        case 0:
                            $a = true;
                            break;

                        default:
                            $a = false;
                    }

                    if ($a) {
                        echo "cool";
                    }
===expect===
UnusedVariable
===ignore===
TODO
