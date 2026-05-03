===description===
callableObjectWithInvalidStringArgument
===file===
<?php
                    /**
                     * @param object&callable(string):void $object
                     */
                    function takesCallableObject(object $object): void {
                        $object(true);
                    }
                
===expect===
InvalidArgument
===ignore===
TODO
