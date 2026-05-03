===description===
conditionalForeachWithUnusedValue
===file===
<?php
                    if (rand(0, 1) > 0) {
                        foreach ([1, 2, 3] as $val) {}
                    }
                
===expect===
UnusedForeachValue
===ignore===
TODO
