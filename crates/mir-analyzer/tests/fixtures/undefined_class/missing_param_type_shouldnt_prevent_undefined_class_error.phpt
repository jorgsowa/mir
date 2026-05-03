===description===
missingParamTypeShouldntPreventUndefinedClassError
===file===
<?php
                    /** @psalm-suppress MissingParamType */
                    function foo($s = Foo::BAR) : void {}
===expect===
UndefinedClass
===ignore===
TODO
