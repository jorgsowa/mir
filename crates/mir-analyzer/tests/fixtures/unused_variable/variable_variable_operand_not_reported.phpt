===description===
variable-variable operand should be marked as read

===file===
<?php
$key = 'value';
echo $$key;
===expect===
