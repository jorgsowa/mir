===description===
wrongCaseClass
===file===
<?php
                    class Foo {}
                    (new foo());
===expect===
InvalidClass
===ignore===
TODO
