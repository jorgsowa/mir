===description===
noParentInAttributeOnClassWithoutParent
===file===
<?php
                    #[Attribute]
                    class SomeAttr
                    {
                        /** @param class-string $class */
                        public function __construct(string $class) {}
                    }

                    #[SomeAttr(parent::class)]
                    class A {}
                
===expect===
ParentNotFound
===ignore===
TODO
