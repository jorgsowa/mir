===description===
Renaming a class reports references to the old name as undefined.
===file:Box.php===
<?php
class Box {}
===file:Use.php===
<?php
function run(): void { new Box(); }
===expect===
<<none>>
===edit:Box.php===
<?php
class Crate {}
===expect===
Use.php: UndefinedClass@2:27-2:30: Class Box does not exist
