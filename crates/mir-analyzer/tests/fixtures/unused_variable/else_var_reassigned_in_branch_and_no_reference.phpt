===description===
elseVarReassignedInBranchAndNoReference
===file===
<?php
                    $a = true;

                    if (rand(0, 1)) {
                        // do nothing
                    } else {
                        $a = false;
                    }
===expect===
UnusedVariable
===ignore===
TODO
