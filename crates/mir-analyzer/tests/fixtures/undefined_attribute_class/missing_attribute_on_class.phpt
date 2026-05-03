===description===
missingAttributeOnClass
===file===
<?php
                    use FooBarPure;

                    #[Pure]
                    class Video {}
===expect===
UndefinedAttributeClass
===ignore===
TODO
