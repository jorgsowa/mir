===description===
Calling a function with correct casing is not reported.
===file===
<?php
function myFunc(): void {}
myFunc();
===expect===
