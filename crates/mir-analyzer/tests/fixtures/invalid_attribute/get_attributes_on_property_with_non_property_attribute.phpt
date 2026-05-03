===description===
getAttributesOnPropertyWithNonPropertyAttribute
===file===
<?php
                    #[Attribute(Attribute::TARGET_CLASS)]
                    class Attr {}

                    class Foo
                    {
                        public string $bar = "baz";
                    }

                    $r = new ReflectionProperty(Foo::class, "bar");
                    $r->getAttributes(Attr::class);
                
===expect===
InvalidAttribute
===ignore===
TODO
