===description===
Undefined variable in interpolated string
===file===
<?php
fn(): string => "$a";
//               ^^ UndefinedVariable: Variable $a is not defined
                
===expect===
