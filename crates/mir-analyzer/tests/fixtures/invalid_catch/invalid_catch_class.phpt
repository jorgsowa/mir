===description===
invalidCatchClass
===file===
<?php
                    class A {}
                    try {
                        $worked = true;
                    }
                    catch (A $e) {}
===expect===
InvalidCatch
===ignore===
TODO
