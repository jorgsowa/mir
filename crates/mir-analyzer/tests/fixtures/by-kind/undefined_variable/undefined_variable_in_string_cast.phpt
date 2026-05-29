===description===
Undefined variable in string cast
===file===
<?php
fn(): string => (string) $a;
                
===expect===
UndefinedVariable@2:26-2:28: Variable $a is not defined
