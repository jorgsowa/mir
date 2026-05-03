===description===
multipleAttributesShowErrors
===file===
<?php
                    #[Attribute(Attribute::TARGET_CLASS)]
                    class Foo {}

                    #[Attribute(Attribute::TARGET_PARAMETER)]
                    class Bar {}

                    #[Foo, Bar]
                    class Baz {}
                
===expect===
InvalidAttribute
===ignore===
TODO
