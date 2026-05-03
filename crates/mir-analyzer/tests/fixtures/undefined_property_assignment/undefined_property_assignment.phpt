===description===
undefinedPropertyAssignment
===file===
<?php
                    class A {
                    }

                    (new A)->foo = "cool";
===expect===
UndefinedPropertyAssignment
===ignore===
TODO
