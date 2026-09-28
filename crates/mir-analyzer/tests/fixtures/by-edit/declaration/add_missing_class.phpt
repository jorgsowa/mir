===description===
Declaring a previously missing class clears the undefined-class issue.
===file:Box.php===
<?php
class Other {}
===file:Use.php===
<?php
function run(): void { new Box(); }
===edit:Box.php===
<?php
class Other {}
class Box {}
===expect===
