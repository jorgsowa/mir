===description===
undefinedClassOneLineInFileAfter
===file===
<?php
                    /**
                     * @psalm-suppress UndefinedClass
                     */
                    new B();
                    new C();
===expect===
UndefinedClass
===ignore===
TODO
