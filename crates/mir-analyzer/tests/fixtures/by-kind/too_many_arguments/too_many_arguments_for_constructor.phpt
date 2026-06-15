===description===
Too many arguments for constructor
===file===
<?php
class A { }
new A("hello");
===expect===
TooManyArguments@3:0-3:14: Too many arguments for A::__construct(): expected 0, got 1
