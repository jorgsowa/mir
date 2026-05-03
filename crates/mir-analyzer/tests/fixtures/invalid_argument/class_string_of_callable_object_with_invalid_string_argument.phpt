===description===
classStringOfCallableObjectWithInvalidStringArgument
===file===
<?php
                    /**
                     * @param class-string<object&callable(string):void> $className
                     */
                    function takesCallableObject(string $className): void {
                        $object = new $className();
                        $object(0);
                    }
===expect===
InvalidArgument
===ignore===
TODO
