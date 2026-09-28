===description===
Declaring a previously missing function clears the undefined-call issue.
===file:Lib.php===
<?php
function other(): void {}
===file:Use.php===
<?php
function run(): void { lib(); }
===edit:Lib.php===
<?php
function other(): void {}
function lib(): void {}
===expect===
