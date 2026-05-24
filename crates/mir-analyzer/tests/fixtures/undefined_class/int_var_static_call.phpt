===description===
intVarStaticCall
===file===
<?php
                    $a = 5;
                    $a::bar();
===expect===
InvalidStringClass@3:20: Dynamic class instantiation requires string or class-string type, got '5'
