===description===
Function name with mixed (not all-caps) wrong casing is detected.
===file===
<?php
function processRequest(): void {}
ProcessRequest();
===expect===
WrongCaseFunction@3:1-3:15: Function name 'ProcessRequest' has incorrect casing; use 'processRequest'
