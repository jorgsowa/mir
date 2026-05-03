===description===
classStringOfCallableObjectWithMissingRequiredStringArgument
===file===
<?php
                    /**
                     * @param class-string<object&callable(string):void> $className
                     */
                    function takesCallableObject(string $className): void {
                        $object = new $className();
                        $object();
                    }
===expect===
TooFewArguments
===ignore===
TODO
