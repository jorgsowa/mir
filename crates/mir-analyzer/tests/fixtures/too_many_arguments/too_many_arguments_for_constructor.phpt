===description===
Too many arguments for constructor
===file===
<?php
class A { }
new A("hello");
===expect===
TooManyArguments
===ignore===
TODO
