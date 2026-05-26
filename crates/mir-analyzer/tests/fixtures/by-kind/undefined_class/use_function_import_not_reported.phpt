===description===
use function import not reported
===config===
suppress=UndefinedFunction,UnusedFunction
===file===
<?php
use function Vendor\Missing\helper;
function run(): void {
    helper();
}
===expect===
