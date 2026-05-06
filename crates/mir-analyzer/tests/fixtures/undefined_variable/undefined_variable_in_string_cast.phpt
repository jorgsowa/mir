===description===
undefinedVariableInStringCast
===file===
<?php
                    fn(): string => (string) $a;
                
===expect===
UndefinedVariable@2:45: Variable $a is not defined
