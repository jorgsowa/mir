===description===
Undefined variable in interpolated string
===file===
<?php
fn(): string => "$a";
                
===expect===
UndefinedVariable@2:18: Variable $a is not defined
