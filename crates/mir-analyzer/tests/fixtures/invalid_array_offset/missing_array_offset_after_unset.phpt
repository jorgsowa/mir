===description===
missingArrayOffsetAfterUnset
===file===
<?php
                    $x = ["a" => "value", "b" => "value"];
                    unset($x["a"]);
                    echo $x["a"];
===expect===
InvalidArrayOffset
===ignore===
TODO
