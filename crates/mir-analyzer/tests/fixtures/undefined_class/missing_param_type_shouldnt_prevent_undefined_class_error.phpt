===description===
missingParamTypeShouldntPreventUndefinedClassError
===file===
<?php
                    /** @psalm-suppress MissingParamType */
                    function foo($s = Foo::BAR) : void {}
===expect===
UndefinedClass@3:38: Class Foo does not exist
UnusedParam@3:33: Parameter $s is never used
