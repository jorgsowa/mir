===description===
A function name containing non-ASCII characters is still subject to ASCII case
checks: only the ASCII letters must match the declaration's casing.
===file===
<?php
function grüFunc(): void {}
GRüFunc();
===expect===
WrongCaseFunction@3:0-3:7: Function name 'GRüFunc' has incorrect casing; use 'grüFunc'
