===description===
reports invalid pass by reference
===file===
<?php
function fill(int &$value): void { $value = 1; }
fill(1 + 2);
===expect===
InvalidPassByReference@3:5: Argument $value of fill() must be passed by reference
