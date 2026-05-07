===description===
attributeTargetArgCannotBeVariable
===file===
<?php
                    $target = 1;

                    #[Attribute($target)]
                    class Foo {}
                
===expect===
UndefinedVariable@4:22: Variable $target is not defined
===ignore===
TODO
