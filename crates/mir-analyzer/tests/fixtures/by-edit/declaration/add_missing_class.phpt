===description===
Declaring a previously missing class clears the undefined-class issue.
===file:Box.php===
<?php
class Other {}
===file:Use.php===
<?php
function run(): void { new Box(); }
===expect===
Use.php: UndefinedClass@2:27-2:30: Class Box does not exist
===edit:Box.php===
<?php
class Other {}
class Box {}
===expect===
