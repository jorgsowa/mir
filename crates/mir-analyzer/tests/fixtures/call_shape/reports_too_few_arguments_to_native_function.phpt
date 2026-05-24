===description===
reports too few arguments to native function
===file===
<?php
str_repeat('x');
===expect===
TooFewArguments@2:1: Too few arguments for str_repeat(): expected 2, got 1
