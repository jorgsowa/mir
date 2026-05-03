===description===
tooManyArgumentsForConstructor
===file===
<?php
                  class A { }
                  new A("hello");
===expect===
TooManyArguments
===ignore===
TODO
