===description===
withoutHasTypeCall
===file===
<?php
                    $method = new ReflectionMethod(stdClass::class);
                    $parameters = $method->getParameters();
                    foreach ($parameters as $parameter) {
                        $parameter->getType()->__toString();
                    }
===expect===
PossiblyNullReference
===ignore===
TODO
