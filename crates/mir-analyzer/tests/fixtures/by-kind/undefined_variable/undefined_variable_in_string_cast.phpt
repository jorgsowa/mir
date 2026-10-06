===description===
Undefined variable in string cast
===file===
<?php
fn(): string => (string) $a;
//                       ^^ UndefinedVariable: Variable $a is not defined
                
