===description===
static call via bare FQN without use statement produces no error
===file:Helper.php===
<?php
class Helper {
    public static function go(): void {}
}
===file:Caller.php===
<?php
function call_it(): void {
    \Helper::go();
}
===expect===
