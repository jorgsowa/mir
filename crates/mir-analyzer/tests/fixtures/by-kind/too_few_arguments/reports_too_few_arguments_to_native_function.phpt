===description===
reports too few arguments to native function
===file===
<?php
str_repeat('x');
//<^^^^^^^^^^^^^^^ TooFewArguments: Too few arguments for str_repeat(): expected 2, got 1
