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
FEATURE: PHP attributes only accept constant expressions, not variables. Should report InvalidAttributeArgument instead of UndefinedVariable once attribute validation is implemented.
