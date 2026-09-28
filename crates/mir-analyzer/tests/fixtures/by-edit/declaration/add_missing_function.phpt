===description===
Declaring a previously missing function clears the undefined-call issue.
===file:Lib.php===
<?php
function other(): void {}
===file:Use.php===
<?php
function run(): void { lib(); }
===expect===
Use.php: UndefinedFunction@2:23-2:28: Function lib() is not defined
===edit:Lib.php===
<?php
function other(): void {}
function lib(): void {}
===expect===
<<none>>
