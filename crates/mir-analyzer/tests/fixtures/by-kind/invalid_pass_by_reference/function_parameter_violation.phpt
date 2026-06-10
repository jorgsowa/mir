===description===
Function parameter violation
===ignore===
TODO
===file===
<?php
/** @return void */
function changeInt(int &$a) {
  $a = "hello";
}
===expect===
