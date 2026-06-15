===description===
Undefined variable in string cast
===file===
<?php
fn(): string => (string) $a;
                
===expect===
UndefinedVariable@2:25-2:27: Variable $a is not defined
