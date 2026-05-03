===description===
detectUnusedVariableReassignedInIfFollowedByTryInsideForeachLoop
===file===
<?php
                    $user_id = 0;
                    $user = null;

                    if (rand(0, 1)) {
                        $user_id = rand(0, 1);
                        $user = $user_id;
                    }

                    if ($user !== null && $user !== 0) {
                        $a = 0;
                        foreach ([1, 2, 3] as $i) {
                            $a += $i;
                            try {} catch (Exception $e) {}
                        }
                        echo $i;
                    }
===expect===
UnusedVariable
===ignore===
TODO
