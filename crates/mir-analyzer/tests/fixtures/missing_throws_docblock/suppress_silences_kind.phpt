===description===
suppress=MissingThrowsDocblock drops every diagnostic of that kind from the result
===config===
suppress=MissingThrowsDocblock,UnusedFunction
===file===
<?php
function riskyOperation(): void {
    throw new \Exception('fail');
}
===expect===
