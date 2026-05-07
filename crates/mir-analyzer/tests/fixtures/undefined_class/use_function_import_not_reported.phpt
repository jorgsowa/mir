===description===
use function import not reported
===file===
<?php
use function Vendor\Missing\helper;
function run(): void {
    helper();
}
===expect===
UndefinedFunction@4:4: Function helper() is not defined
