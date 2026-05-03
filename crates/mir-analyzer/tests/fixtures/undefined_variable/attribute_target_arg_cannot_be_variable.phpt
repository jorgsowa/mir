===description===
attributeTargetArgCannotBeVariable
===file===
<?php
                    $target = 1;

                    #[Attribute($target)]
                    class Foo {}
                
===expect===
UndefinedVariable
===ignore===
TODO
