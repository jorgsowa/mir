===description===
Function name with mixed (not all-caps) wrong casing is detected.
===file===
<?php
function processRequest(): void {}
ProcessRequest();
//<^^^^^^^^^^^^^^ WrongCaseFunction: Function name 'ProcessRequest' has incorrect casing; use 'processRequest'
===expect===
