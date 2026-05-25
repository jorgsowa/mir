===description===
Object callable intersection correct
===file===
<?php
/**
 * @param object&callable(string):void $obj
 */
function test(object $obj): void {
    $obj('hello');
}

===expect===
