===description===
referenceReuseForeachValue
===file===
<?php
                    /** @var array<int> */
                    $arr = [];

                    foreach ($arr as &$var) {
                        $var += 1;
                    }

                    $var = "foo";
                
===expect===
ReferenceReusedFromConfusingScope
===ignore===
TODO
