===description===
Possibly null function call
===file===
<?php
$this->foo();
===expect===
InvalidScope@2:0-2:5: $this cannot be used outside of a class
