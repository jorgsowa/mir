===description===
Possibly null function call
===file===
<?php
$this->foo();
===expect===
InvalidScope@2:1-2:6: $this cannot be used outside of a class
