===description===
functionParameterViolation
===file===
<?php
/** @return void */
function changeInt(int &$a) {
  $a = "hello";
}
===expect===
ReferenceConstraintViolation
===ignore===
TODO
