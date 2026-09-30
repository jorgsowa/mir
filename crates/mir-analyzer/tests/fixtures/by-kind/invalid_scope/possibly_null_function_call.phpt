===description===
Possibly null function call
===file===
<?php
$this->foo();
//<^^^^^ InvalidScope: $this cannot be used outside of a class
===expect===
