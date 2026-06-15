===description===
Undefined variable in interpolated string
===file===
<?php
fn(): string => "$a";
                
===expect===
UndefinedVariable@2:17-2:19: Variable $a is not defined
