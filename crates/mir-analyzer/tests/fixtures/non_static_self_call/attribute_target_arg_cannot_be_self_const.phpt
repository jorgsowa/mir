===description===
attributeTargetArgCannotBeSelfConst
===file===
<?php
                    #[Attribute(self::BAR)]
                    class Foo
                    {
                        public const BAR = 1;
                    }
                
===expect===
NonStaticSelfCall
===ignore===
TODO
