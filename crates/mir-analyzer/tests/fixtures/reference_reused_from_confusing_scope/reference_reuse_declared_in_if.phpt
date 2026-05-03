===description===
referenceReuseDeclaredInIf
===file===
<?php
                    /** @var array<int> */
                    $arr = [];

                    if (isset($arr[0])) {
                        $var = &$arr[0];
                        $var += 1;
                    }

                    $var = "foo";
                
===expect===
ReferenceReusedFromConfusingScope
===ignore===
TODO
