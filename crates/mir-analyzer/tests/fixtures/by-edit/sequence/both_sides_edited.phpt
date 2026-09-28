===description===
Editing a callee and its caller together checks the new pair against each other.
===file:Lib.php===
<?php
function lib(): int { return 1; }
===file:Use.php===
<?php
function run(): int { return lib(); }
===expect===
===edit:Lib.php===
<?php
function lib(): string { return ''; }
===edit:Use.php===
<?php
function run(): string { return lib(); }
===expect===
