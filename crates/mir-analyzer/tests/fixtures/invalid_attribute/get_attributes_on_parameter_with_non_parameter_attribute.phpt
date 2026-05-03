===description===
getAttributesOnParameterWithNonParameterAttribute
===file===
<?php
                    #[Attribute(Attribute::TARGET_PROPERTY)]
                    class Attr {}

                    function foo(int $bar): void {}

                    $r = new ReflectionParameter("foo", "bar");
                    $r->getAttributes(Attr::class);
                
===expect===
InvalidAttribute
===ignore===
TODO
