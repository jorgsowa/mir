===description===
undefinedVariableInInterpolatedString
===file===
<?php
fn(): string => "$a";
                
===expect===
UndefinedVariable@2:17: Variable $a is not defined
