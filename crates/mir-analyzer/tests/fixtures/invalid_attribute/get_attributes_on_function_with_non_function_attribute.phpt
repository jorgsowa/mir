===description===
getAttributesOnFunctionWithNonFunctionAttribute
===file===
<?php
                    #[Attribute(Attribute::TARGET_PROPERTY)]
                    class Attr {}

                    function foo(): void {}

                    /** @psalm-suppress InvalidArgument */
                    $r = new ReflectionFunction("foo");
                    $r->getAttributes(Attr::class);
                
===expect===
InvalidAttribute
===ignore===
TODO
