===description===
reports too many arguments to native function
===file===
<?php
strlen('hello', 'extra');
===expect===
TooManyArguments@2:16: Too many arguments for strlen(): expected 1, got 2
