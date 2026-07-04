===description===
Default above case that breaks
===file===
<?php
function foo(string $a) : string {
  switch ($a) {
    case "a":
      return "hello";

    default:
    case "b":
      break;

    case "c":
      return "goodbye";
  }
}
===expect===
InvalidReturnType@2:33-14:1: Return type 'void' is not compatible with declared 'string'
