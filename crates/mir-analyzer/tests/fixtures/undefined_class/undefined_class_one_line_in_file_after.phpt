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
UndefinedClass@6:24: Class C does not exist
