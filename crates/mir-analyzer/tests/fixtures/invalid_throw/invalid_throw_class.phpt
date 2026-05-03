===description===
invalidThrowClass
===file===
<?php
                    class A {}
                    throw new A();
===expect===
InvalidThrow
===ignore===
TODO
