===description===
Removing a function reports its callers as undefined calls.
===file:Lib.php===
<?php
function lib(): void {}
function other(): void {}
===file:Use.php===
<?php
function run(): void { lib(); }
===edit:Lib.php===
<?php
function other(): void {}
===expect===
Use.php: UndefinedFunction@2:23-2:28: Function lib() is not defined
