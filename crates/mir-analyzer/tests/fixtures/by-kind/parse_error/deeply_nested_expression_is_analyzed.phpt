===description===
Expressions nested past the old 50-level parser limit parse and are analyzed
instead of being rejected with a nesting-depth error.
===file===
<?php
echo (((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((($undefinedVar))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))));
//                                                                                                       ^^^^^^^^^^^^^ UndefinedVariable: Variable $undefinedVar is not defined
