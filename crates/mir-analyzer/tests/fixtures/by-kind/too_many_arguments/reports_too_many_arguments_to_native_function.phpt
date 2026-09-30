===description===
reports too many arguments to native function
===file===
<?php
strlen('hello', 'extra');
//              ^^^^^^^ TooManyArguments: Too many arguments for strlen(): expected 1, got 2
===expect===
