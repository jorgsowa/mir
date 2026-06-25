===description===
@param description text that contains a $var reference must not bleed into the
type string or replace the declared param name with the $var from the description.
===file===
<?php

/**
 * @param bool $flag Whether the $option should be checked
 */
function checkFlag($flag): void {
    /** @mir-check $flag is bool */
    echo $flag;
}

/**
 * @param string $name The name of the $item to look up
 * @param int $count How many $items to process
 */
function process($name, $count): void {
    /** @mir-check $name is string */
    /** @mir-check $count is int */
    echo $name . $count;
}

===expect===
