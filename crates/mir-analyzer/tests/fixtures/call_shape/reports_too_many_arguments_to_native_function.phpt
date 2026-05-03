===description===
reports too many arguments to native function
===file===
<?php
strlen('hello', 'extra');
===expect===
TooManyArguments: Too many arguments for strlen(): expected 1, got 2
===ignore===
TODO
