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
UndefinedClass@5:24: Class B does not exist
UndefinedClass@6:24: Class C does not exist
